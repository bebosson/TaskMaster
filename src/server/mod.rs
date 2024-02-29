use bincode::{deserialize, serialize};
use std::env;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::{self, spawn, JoinHandle};
use std::time::Duration;

pub mod conf;
// pub mod quit;
pub mod file;
pub mod log;
pub mod loop_exec;
pub mod parse;
pub mod proc;
pub mod task;
mod tool;

use conf::Taskmaster;
use proc::Proc;
use share_structures::{CallOn, Request, Response, State};

pub fn spawn_thread_up_ex(arc_mut: Arc<Mutex<Taskmaster>>) -> JoinHandle<()> {
    let task_clone = arc_mut.clone();

    let t = thread::spawn(move || loop {
        let mut task_mut = task_clone.lock().unwrap();
        task_mut.update_and_exec();
        thread::sleep(Duration::from_millis(100));
        if task_mut.all_task_stable() == true {
            break;
        }
    });

    t
}

fn handle_client(mut stream: TcpStream, task: Arc<Mutex<Taskmaster>>) {
    use CallOn::*;

    let mut data = [0 as u8; 1500];

    while match stream.read(&mut data) {
        Ok(size) => {
            // receive request and send response
            if size > 0 && size <= 1500 {
                let req: Request = deserialize(&data).unwrap();
                let res: Response;
                match req.cmd {
                    // Fill Response Struct
                    Start(proc_name) => {
                        res = task.lock().unwrap().start(proc_name);
                    }
                    Stop(proc_name) => {
                        res = task.lock().unwrap().stop(proc_name);
                    }
                    Restart(proc_name) => {
                        res = task.lock().unwrap().restart(proc_name);
                    }
                    Status => {
                        res = task.lock().unwrap().status();
                    }
                    Reload => {
                        res = task.lock().unwrap().reload();
                    }
                }
                let res: Vec<u8> = serialize(&res).unwrap();
                stream.write(res.as_slice()).unwrap();
            }
            true
        }
        Err(_) => {
            println!(
                "An error occurred, terminating connection with {}",
                stream.peer_addr().unwrap()
            );
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {
        //end thread at the end of Req
        break;
    }
}

fn run_server(taskmaster: Arc<Mutex<Taskmaster>>) {
    let listener = TcpListener::bind("127.0.0.1:9003").unwrap();

    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port 9003");
    for stream in listener.incoming() {
        let taskmaster = taskmaster.clone();
        match stream {
            Ok(stream) => {
                // New connection
                thread::spawn(move || {
                    // connection succeeded
                    handle_client(stream, taskmaster);
                });
            }
            Err(e) => {
                println!("Error: {}", e);
                /* connection failed */
            }
        }
    }
    // close the socket server
    drop(listener);
}

/// TODO check with config file if process struct shoult be removed from Task.process_list
fn drain_proc(process: &mut Vec<Proc>, index_lst: &mut Vec<usize>, numprocs: Option<i64>) {
    let numprocs = numprocs.unwrap() as usize;
    index_lst.reverse();

    for i in index_lst.clone() {
        if process.len() > numprocs {
            process.remove(i);
        }
    }

    index_lst.clear();
}

fn process_handler(taskmaster: Arc<Mutex<Taskmaster>>) {
    let mut app = taskmaster.lock().unwrap();
    let mut to_rm_queue = vec![];

    for task in &mut app.task_lst {
        for (p_index, proc) in &mut task.process_lst.iter_mut().enumerate() {
            match &mut proc.child {
                Some(child) => {
                    match child.try_wait() {
                        Ok(Some(exit_status)) => {
                            // Process has exited
                            if proc.state == State::STOPPING {
                                proc.change_to_stopped();
                            } else {
                                proc.change_to_exited();
                            }
                            proc.exit_error = exit_status.code();
                            proc.test_autorestart(task.parse_file.clone(), exit_status);
                        }
                        Ok(None) => {
                            // Process is running
                            if proc.state == State::STARTING {
                                proc.change_to_running();
                            } else if proc.state == State::STOPPING {
                                let duration = proc.started_time.unwrap().elapsed();
                                let exp_duration = Duration::from_secs(
                                    task.parse_file.stopwaitsecs.unwrap().try_into().unwrap(),
                                );
                                if duration > exp_duration {
                                    // force kill
                                    child.kill().expect("proc wasn't running");
                                }
                                proc.change_state(State::STOPPING);
                            }
                        }
                        Err(e) => println!("error attempting to wait: {e}"),
                    }
                }
                None => {
                    if proc.name == None {
                        to_rm_queue.push(p_index);
                    }
                }
            }
        }
        // remove proc
        drain_proc(
            &mut task.process_lst,
            &mut to_rm_queue,
            task.parse_file.numprocs,
        );
    }
    // remove empty Task
    app.task_lst.retain(|task| !task.is_active == false);
}

fn supervisor(taskmaster: Arc<Mutex<Taskmaster>>) {
    // Surveillance of process
    spawn(move || loop {
        process_handler(taskmaster.clone());
        thread::sleep(Duration::from_secs(2));
    });
}

fn main() {
    let path = env::args()
        .nth(1)
        .expect("\nUsage :\n\tcargo run --bin server -- CONFIG_FILE\n\n");

    let mut app = Taskmaster::new(&path);
    app.set_all_command();
    let app = Arc::new(Mutex::new(app));
    let t = spawn_thread_up_ex(app.clone());
    let _ = t.join().unwrap();
    supervisor(app.clone());
    run_server(app.clone());
}
