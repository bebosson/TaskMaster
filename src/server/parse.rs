use nix::sys::{signal::Signal, stat::Mode};
use yaml_rust::Yaml;

use super::{file::FileLog, loop_exec::Autorestart, tool::parse_to_string};

#[derive(Debug, Clone)]
pub struct File {
    pub path_command: Option<String>,
    pub args: Option<Vec<String>>,
    pub name: Option<String>,
    pub numprocs: Option<i64>,
    pub autostart: Option<bool>,
    pub autorestart: Option<Autorestart>,
    pub exitcodes: Vec<i32>,
    pub starttime: Option<i64>,    //unsigned
    pub startretries: Option<i64>, //unsigned
    pub stopsignal: Signal,
    pub stopwaitsecs: Option<i64>,
    pub directory: Option<String>,
    pub umask: Option<Mode>, //u8
    pub bool_stdout: bool,
    pub bool_stderr: bool,
    pub file_log: Option<FileLog>,
}

impl File {
    pub fn from_yaml(yaml_file: &Yaml) -> Self {
        let opt_name = parse_to_string(yaml_file["name"].as_str());
        // C'est degueulasse !!! TODO
        let exitcodes: Vec<i32> = yaml_file["exitcodes"]
            .clone()
            .into_iter()
            .map(|value| value.as_i64().unwrap_or(0) as i32)
            .collect();

        File {
            path_command: parse_to_string(yaml_file["command"].as_str()),
            name: opt_name.clone(),
            numprocs: yaml_file["numprocs"].as_i64(),
            autostart: yaml_file["autostart"].as_bool(),
            autorestart: parse_autorestart(
                yaml_file["autorestart"].as_bool(),
                parse_to_string(yaml_file["autorestart"].as_str()),
            ),
            exitcodes,
            stopwaitsecs: yaml_file["stopwaitsecs"].as_i64(),
            starttime: yaml_file["starttime"].as_i64(),
            stopsignal: parse_signal(yaml_file["stopsignal"].as_str()),
            startretries: yaml_file["startretries"].as_i64(),
            directory: parse_to_string(yaml_file["directory"].as_str()),
            umask: mode_from_string(yaml_file["umask"].as_str()),
            args: None,
            bool_stdout: yaml_file["stdout"].is_badvalue(),
            bool_stderr: yaml_file["stderr"].is_badvalue(),
            file_log: None,
        }
    }

    pub fn prgm_is_launchable(&self) -> bool {
        self.name.is_some() && self.path_command.is_some()
    }

    pub fn get_args(&mut self) {
        let split = self.path_command.as_ref().unwrap().split(" ");
        self.args.get_or_insert(Vec::new());
        split
            .enumerate()
            .map(|x| self.args.as_mut().unwrap().push(x.1.to_string()))
            .for_each(drop);
        let first = self.args.as_deref_mut().unwrap().first().unwrap();
        let command: &mut String = self.path_command.as_mut().unwrap();
        *command = first.to_string();
        self.args.as_mut().unwrap().remove(0);
    }

    pub fn init_args(&mut self) {
        let x = self.path_command.as_ref().unwrap().split(" ");
        match x.count() {
            1.. => self.get_args(),
            _ => self.args = None,
        }
    }

    pub fn set_conf_default(&mut self) {
        self.set_numprocs_default();
        self.set_autostart_default();
        self.set_autorestart_default();
        self.set_exitcode_default();
        self.set_start_retries_default();
        self.set_start_retries_default();
        self.set_startsecs_default();
        self.set_stopwaitsecs_default();
        self.set_umask_default();
    }

    pub fn set_numprocs_default(&mut self) {
        if self.numprocs == None {
            self.numprocs = Some(1);
        }
    }

    pub fn set_autostart_default(&mut self) {
        if self.autostart == None {
            self.autostart = Some(true);
        }
    }

    pub fn set_autorestart_default(&mut self) {
        if self.autorestart == None {
            self.autorestart = Some(Autorestart::Never); //tmp (unexpected => string but better if its u8 like 0?)
        }
    }

    pub fn set_exitcode_default(&mut self) {
        self.exitcodes = vec![0];
    }

    pub fn set_start_retries_default(&mut self) {
        if self.startretries == None {
            self.startretries = Some(3);
        }
    }

    pub fn set_startsecs_default(&mut self) {
        if self.starttime == None {
            self.starttime = Some(0);
        }
    }

    pub fn set_stopwaitsecs_default(&mut self) {
        if self.stopwaitsecs == None {
            self.stopwaitsecs = Some(10);
        }
    }

    pub fn set_umask_default(&mut self) {
        if self.umask == None {
            self.umask = mode_from_string(Some(&"022".to_string()));
        }
    }
}

pub fn mode_from_string(str: Option<&str>) -> Option<Mode> {
    let param: u32;
    match str {
        Some(str) => {
            param = u32::from_str_radix(&str, 8).expect("wesh");
            Mode::from_bits(param)
        }
        None => None,
    }
}

pub fn parse_autorestart(
    opt_bool: Option<bool>,
    opt_string: Option<String>,
) -> Option<Autorestart> {
    let y: Option<Autorestart>;
    let parse = (opt_bool, opt_string);

    match parse {
        (None, None) => y = None,
        (None, Some(_)) => y = Some(Autorestart::OnError),
        (Some(true), _) => y = Some(Autorestart::Always),
        (Some(false), _) => y = Some(Autorestart::Never),
    }
    y
}

pub fn parse_signal(sig: Option<&str>) -> Signal {
    match sig {
        Some(signal) => match signal {
            "TERM" => Signal::SIGTERM,
            "HUP" => Signal::SIGHUP,
            "INT" => Signal::SIGINT,
            "QUIT" => Signal::SIGQUIT,
            "KILL" => Signal::SIGKILL,
            _ => Signal::SIGTERM,
        },
        None => Signal::SIGTERM,
    }
}
