use chrono::{DateTime, Local};
use nix::sys::{signal::Signal, stat};
use nix::unistd::Pid;
use std::os::unix::process::ExitStatusExt;
use std::thread;
use std::time::Duration;
use std::{
    fmt,
    os::unix::process::CommandExt,
    process::{Child, Command, ExitStatus},
    time::{Instant, SystemTime},
};

use crate::loop_exec::Autorestart;

use super::{
    file::FileLog,
    loop_exec::{always_true, test_autorestart, test_autostart, test_time_starting, LoopRestart},
    parse::File,
    tool::DurationDate,
};
use share_structures::State;

pub struct Proc {
    pub name: Option<String>,
    pub command: Option<Command>,
    pub child: Option<Child>,
    pub exit_error: Option<i32>,
    pub sys_time: Option<SystemTime>,
    pub sys_date: Option<DateTime<Local>>,
    pub s_file: FileLog,
    pub state: State,
    pub pid: Option<u32>,
    pub description: String,
    pub started_time: Option<Instant>,
    pub nbr_restart: i64,
    pub loop_action_true: Option<fn(&mut Proc)>,
    pub loop_action_false: Option<fn(&mut Proc)>,
    pub loop_test_file_config: Option<fn(&File, &Proc) -> bool>,
    pub loop_bool: bool,
    pub stopsignal: Signal,
    
    pub exp_exit: Vec<i32>,
    pub exp_duration: Duration,
}

impl fmt::Debug for Proc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Proc")
            .field("name", &self.name)
            .field("pid", &self.pid)
            .field("state", &self.state)
            .field("stopsignal", &self.stopsignal)
            .field("exit", &self.exit_error)
            .finish()
    }
}

impl Proc {
    // ______________________ Actions _________________________
    pub fn start(&mut self) -> Result<String, String> {
        if self.state == State::STOPPED || self.state == State::EXITED {
            if let Ok(child) = self.command.as_mut().expect("Start").spawn() {
                println!("INFO spawned: {} with {}", self.get_name(), child.id());
                self.state = State::STARTING;
                self.child.get_or_insert(child);
                Ok(format!("{}: started", self.get_name()))
            } else {
                println!("Command didn't start");
                Err(format!("Command didn't start"))
            }
        } else {
            Err(format!("ERROR (already started)"))
        }
    }

    pub fn stop(&mut self) -> Result<String, String> {
        if self.state == State::RUNNING {
            match &mut self.child {
                Some(child) => {
                    let pid = Pid::from_raw(child.id().try_into().unwrap());
                    self.state = State::STOPPING;
                    self.started_time = Some(Instant::now());
                    nix::sys::signal::kill(pid, self.stopsignal).expect("proc wasn't running");
                    return Ok(format!("{}: stopped", self.get_name()));
                }
                None => Err(format!("No child found")),
            }
        } else {
            Err(format!("{}: ERROR (not running)", self.get_name()))
        }
    }

    pub fn restart(&mut self) -> Result<String, String> {
        match self.stop() {
            Ok(stopped) => {
                self.change_to_stopped();
                match self.start() {
                    Ok(started) => Ok(format!("{stopped}\n\r{started}")),
                    Err(e_start) => Err(format!("{stopped}\n\r{e_start}")),
                }
            }
            Err(e_stopped) => match self.start() {
                Ok(started) => Ok(format!("{e_stopped}\n\r{started}")),
                Err(e_start) => Err(format!("{e_stopped}\n\r{e_start}")),
            },
        }
    }

    pub fn exit_or_running(&mut self) -> bool {
        // rename : refresh states process and description
        if let Some(child) = &mut self.child {
            match child.try_wait() {
                Ok(Some(_)) => return true,
                Ok(None) => return false,
                Err(e) => {
                    panic!("error attempting to wait: {e}");
                }
            }
        } else {
            panic!("Child doesn't exist");
        }
    }

    pub fn exit_process(&mut self) {
        // rename : refresh states process and description
        if let Some(child) = &mut self.child {
            match child.try_wait() {
                Ok(Some(status)) => {
                    self.description = format!("exited with: {}", status);
                    self.child = None;
                    self.started_time = None;
                    self.exit_error = status.code();
                    self.change_state(State::EXITED);
                }
                Ok(None) => {
                    println!("status not ready yet, let's really wait");
                    let res = child.wait();
                    println!("result: {res:?}");
                }
                Err(e) => println!("error attempting to wait: {e}"),
            }
        }
    }

    pub fn test_autorestart(&mut self, config: File, exit_status: ExitStatus) {
        match config.autorestart {
            Some(instruction) => match instruction {
                Autorestart::Always => {
                    let _ = self.restart();
                }
                Autorestart::OnError => {
                    let code = exit_status.code().unwrap_or_default();
                    let signal = exit_status.signal().unwrap_or_default();

                    if !self.exp_exit.contains(&code) || self.stopsignal as i32 != signal {
                        let _ = self.restart();
                    }
                }
                Autorestart::Never => (),
            },
            None => (),
        }
    }

    // _____________________ Getter ___________________________
    pub fn get_name(&self) -> String {
        match &self.name {
            Some(name) => String::from(name),
            None => String::new(),
        }
    }

    pub fn get_current_description(&self) -> String {
        if self.state == State::RUNNING {
            format!(
                "pid {} uptime: {}",
                self.pid.unwrap(),
                self.started_time.unwrap().elapsed().durationdate()
            )
        } else {
            self.description.clone()
        }
    }

    pub fn get_state(&self) -> State {
        self.state.clone()
    }

    pub fn get_status(&self) {
        println!(
            "proc_name = {} proc_status {:?}",
            self.name.as_ref().unwrap(),
            self.state
        )
    }

    pub fn get_exit_status(&mut self) -> Option<ExitStatus> {
        match &mut self.child {
            Some(child) => match child.try_wait() {
                Ok(Some(status)) => Some(status),
                _ => None,
            },
            None => None,
        }
    }

    // ____________________ Setter ____________________________
    pub fn new(command_arg: Command, file_log: FileLog, opt_name: Option<String>) -> Self {
        Proc {
            name: opt_name,
            command: Some(command_arg),
            child: None,
            exit_error: None,
            s_file: file_log,
            sys_time: Some(SystemTime::now().into()), //1---- so not now
            sys_date: Some(Local::now()),
            state: State::STOPPED,
            pid: None,
            description: "Not Started".to_string(),
            started_time: None,
            loop_action_false: None,
            loop_action_true: None,
            loop_test_file_config: None,
            loop_bool: false,
            nbr_restart: 0,
            stopsignal: Signal::SIGTERM,
            exp_exit: vec![0],
            exp_duration: Duration::new(0, 0),
        }
    }

    pub fn change_state(&mut self, state: State) {
        match state {
            State::STOPPED => println!("{}", self.stopped_log()),
            State::STARTING => println!("{}", self.spawn_log()),
            State::RUNNING => println!("{}", self.running_log()),
            State::STOPPING => println!("{}", self.stopping_log()),
            State::BACKOFF => {}
            State::EXITED => println!("{}", self.exit_log()),
            State::FATAL => println!("{}", self.gaveup_log()),
            State::UNKNOWN => todo!(),
        }
        self.state = state;
    }

    pub fn change_to_backoff(&mut self) {
        self.change_state(State::BACKOFF);
    }

    pub fn change_to_fatal(&mut self) {
        self.change_state(State::FATAL);
    }

    pub fn change_to_running(&mut self) {
        if let Some(child) = &self.child {
            self.pid.get_or_insert(child.id());
        };
        self.started_time = Some(Instant::now());
        // println!("{:?}", self.exp_duration);
        // thread::sleep(self.exp_duration);
        self.change_state(State::RUNNING);
    }

    pub fn change_to_exited(&mut self) {
        self.change_state(State::EXITED);
        self.child = None;
        self.pid = None;
        self.description = format!("{}", Local::now().format("%b %d %H:%M %p"));
    }

    pub fn change_to_stopped(&mut self) {
        self.change_state(State::STOPPED);
        self.child = None;
        self.pid = None;
        self.description = format!("{}", Local::now().format("%b %d %H:%M %p"));
    }

    pub fn update_methods(&mut self) {
        match self.state {
            State::STOPPED => {
                self.loop_test_file_config = Some(test_autostart);
                self.loop_action_true = Some(Proc::start_loop); // Stopped -> Starting
                self.loop_action_false = None;
            }
            State::STARTING => {
                match self.exit_or_running() {
                    true => {
                        // proc exit
                        self.loop_test_file_config = Some(always_true);
                        self.loop_action_true = Some(Proc::exit_process); // Starting -> Exited
                        self.loop_action_false = None
                    }
                    false => {
                        // proc survive
                        self.loop_test_file_config = Some(test_time_starting);
                        self.loop_action_true = Some(Proc::change_to_running); // Starting -> Running
                        self.loop_action_false = None
                    }
                }
            }
            State::RUNNING => {
                self.loop_test_file_config = None;
                self.loop_action_true = None;
                self.loop_action_false = None
            }
            State::STOPPING => {
                self.loop_test_file_config = None;
                self.loop_action_true = None;
                self.loop_action_false = None
            }
            State::BACKOFF => {
                self.loop_test_file_config = Some(always_true);
                self.loop_action_true = Some(Proc::start_loop); // Backoff -> Starting
                self.loop_action_false = None;
            }
            State::EXITED => {
                self.loop_test_file_config = Some(test_autorestart);
                self.loop_action_true = Some(Proc::change_to_backoff); // Exited -> Backoff
                self.loop_action_false = Some(Proc::change_to_fatal); // Exited -> Fatal
            }
            State::FATAL => {
                self.loop_test_file_config = None;
                self.loop_action_true = None;
                self.loop_action_false = None
            }
            State::UNKNOWN => {
                self.loop_test_file_config = None;
                self.loop_action_true = None;
                self.loop_action_false = None;
            }
        }
    }

    pub fn set_args_task(&mut self, parse_file: &File) {
        if let Some(args) = &parse_file.args {
            if let Some(command) = &mut self.command {
                command.args(args.clone());
            }
        }
    }

    pub fn setup_command_umask(&mut self, parse_file: &File) {
        if let Some(umask) = parse_file.umask {
            unsafe {
                if let Some(command) = &mut self.command {
                    command.pre_exec(move || {
                        stat::umask(umask);
                        Ok(())
                    });
                }
            }
        }
    }
}
