use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::net::TcpStream;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub enum State {
    STOPPED,
    //The process has been stopped due to a stop request or has never been started.
    STARTING,
    //The process is starting due to a start request.
    RUNNING,
    //The process is running
    STOPPING,
    //The process is stopping due to a stop request.
    BACKOFF,
    //The process entered the STARTING state but subsequently exited too quickly (before the time defined in startsecs) to move to the RUNNING state
    EXITED,
    //The process exited from the RUNNING state (expectedly or unexpectedly).
    FATAL,
    //The process could not be started successfully.
    #[default]
    UNKNOWN,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub cmd: CallOn,
    pub success: Result<String, String>,
    pub content: Vec<Prog>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Prog {
    pub proc_name: String,
    pub status: State,
    pub info: String,
}

#[derive(PartialEq, Debug, Serialize, Clone, Deserialize)]
pub enum CallOn {
    Start(String),
    Stop(String),
    Restart(String),
    Status,
    Reload,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Request {
    pub cmd: CallOn,
    pub len: usize,
}

impl Request {
    fn parse(cmd: &String, lst_proc: Vec<String>) -> Result<Self, String> {
        let args: Vec<String> = cmd.split(" ").map(|s| s.to_string()).collect();
        let len = args.len();

        if len <= 2 {
            return match Self::is_command(args, len, lst_proc) {
                Ok(cmd) => Ok(Self { cmd, len }),
                Err(e) => Err(e),
            };
        }
        Err("Too many arguments.".to_string())
    }

    fn proc_exist(proc_requested: String, lst_proc: Vec<String>) -> Result<String, String> {
        match lst_proc.iter().any(|x| x == &proc_requested) {
            true => return Ok(proc_requested),
            false => return Err(format!("{}: ERROR (no such process)", proc_requested)),
        }
    }

    fn is_command(
        command: Vec<String>,
        len: usize,
        proc_list: Vec<String>,
    ) -> Result<CallOn, String> {
        if len == 1 {
            match &command[0][..] {
                "Status" | "status" => Ok(CallOn::Status),
                "Reload" | "reload" => Ok(CallOn::Reload),
                _ => Err(format!("*** Unknown syntax: {}", &command[0])),
            }
        } else {
            match Self::proc_exist(command[1].clone(), proc_list) {
                Ok(proc) => match &command[0][..] {
                    "Start" | "start" => Ok(CallOn::Start(proc)),
                    "Stop" | "stop" => Ok(CallOn::Stop(proc)),
                    "Restart" | "restart" => Ok(CallOn::Restart(proc)),
                    _ => Err(format!("** Unknown syntax: {:?}", command)),
                },
                Err(e) => Err(e),
            }
        }
    }

    pub fn send(command_prompt: &String, proc_list: Vec<String>) -> Result<Response, String> {
        match Self::parse(&command_prompt, proc_list) {
            Ok(cmd) => match TcpStream::connect("127.0.0.1:9003") {
                Ok(mut stream) => {
                    let cmd: Vec<u8> = serialize(&cmd).unwrap();
                    stream.write(cmd.as_slice()).unwrap();
                    return Self::response(stream);
                }
                Err(_e) => return Err(format!("http://localhost:9003 refused connection")),
            },
            Err(e) => Err(e),
        }
    }

    pub fn response(mut stream: TcpStream) -> Result<Response, String> {
        let mut data = [0 as u8; 1500];

        match stream.read(&mut data) {
            Ok(size) => {
                if size <= 1500 {
                    let res: Response = deserialize(&data).unwrap();
                    return Ok(res);
                }
                Err("Response exceed buffer size".to_string())
            }
            Err(e) => Err(format!("{e}")),
        }
    }
}
