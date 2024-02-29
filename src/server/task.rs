use std::{fmt, time::Duration};
use yaml_rust::Yaml;

use crate::loop_exec::exec_loop;

use super::{
    file::{FileLog, Files},
    parse::File,
    proc::Proc,
    tool::n_name,
};

pub struct Task {
    pub name: String,
    pub process_lst: Vec<Proc>,
    pub parse_file: File,
    pub parse_doc_yaml: Yaml,
    pub num_restart: i64,
    pub loop_test: Option<fn(&Task) -> bool>, //possible mais theoriquement nul car 2 proc peuvent ne pas avoir le meme status
    pub is_active: bool,
}

impl fmt::Debug for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Task")
            .field("name", &self.name)
            .field("process_lst", &self.process_lst)
            .field("parse", &self.parse_file)
            .field("yaml", &self.parse_doc_yaml)
            .field("is_active", &self.is_active)
            .finish()
    }
}

impl Task {
    // ______________________ Actions _________________________
    pub fn new(task_config: &Yaml) -> Self {
        Task {
            name: String::new(),
            process_lst: Vec::new(),
            parse_file: File::from_yaml(task_config),
            parse_doc_yaml: task_config.clone(),
            num_restart: 0,
            loop_test: None,
            is_active: true,
            // file_log: Some(FileLog::from_file(task_file, task_name))
        }
    }

    // ____________________ Setter ____________________________
    pub fn set_command(&mut self, name: String) -> i64 {
        let numprocs = self.getnumprocs();
        let lauchable = self.launchable();

        let mut new_proc: Proc;
        let mut filelog = FileLog::from_file(self.parse_doc_yaml.clone(), Some(name.clone()));
        for i in 0..numprocs {
            match lauchable {
                true => {
                    self.parse_file.set_conf_default();
                    new_proc = self.proc_file_log(n_name(name.clone(), i), &i, &mut filelog);
                    new_proc.set_args_task(&self.parse_file);
                    new_proc.setup_command_umask(&self.parse_file);
                    new_proc.exp_exit = self.parse_file.exitcodes.clone();
                    new_proc.stopsignal = self.parse_file.stopsignal;
                    new_proc.exp_duration = Duration::new(
                        self.parse_file.starttime.unwrap_or(0).try_into().unwrap_or(0),
                        0,
                    );
                    self.process_lst.push(new_proc);
                }
                false => {}
            }
        }
        numprocs
    }

    pub fn proc_file_log(&self, n_name: String, i: &i64, filelog: &mut FileLog) -> Proc {
        let mut new_proc: Proc;

        new_proc = self.proc_file_shared_log(&i, filelog, n_name);
        new_proc.files();
        new_proc.redirection();
        new_proc
    }
    // need name, i, filelog

    pub fn launchable(&self) -> bool {
        self.parse_file.prgm_is_launchable()
    }

    // _____________________ Getter ___________________________
    pub fn getnumprocs(&self) -> i64 {
        self.parse_file.numprocs.unwrap_or(1)
    }

    pub fn getnameparse(&self) -> &String {
        self.parse_file.name.as_ref().expect("no name ?")
    }

    pub fn getpathref(&self) -> &String {
        self.parse_file.path_command.as_ref().expect("no path?")
    }

    pub fn getboolfile(&self) -> &File {
        &self.parse_file
    }

    pub fn getautostart(&self) -> bool {
        self.parse_file.autostart.unwrap()
    }

    pub fn getnbrestart(&self) -> i64 {
        self.parse_file.numprocs.unwrap()
    }

    #[allow(unused_must_use)]
    pub fn start_all_proc(&mut self) {
        // need to depend of args numprocs_start and autostart
        for i in &mut self.process_lst {
            i.start();
        }
    }

    pub fn autostart(&self) -> bool {
        self.parse_file.autostart.expect("no autostart?")
    }

    pub fn call_exec_loop(&mut self) {
        let ref_file = &self.parse_file;
        let ref_vec_process = &mut self.process_lst;
        exec_loop(ref_vec_process, &ref_file);
    }

    #[allow(unused_must_use)]
    pub fn stop_all_process(&mut self) {
        for proc in &mut self.process_lst {
            proc.stop();
        }
    }

    #[allow(unused_must_use)]
    pub fn remove_nb_process(&mut self, mut to_be_remove: i64) {
        let mut last = self.process_lst.len() as i32 - 1;

        if to_be_remove > 0 {
            while to_be_remove > 0 {
                self.process_lst[last as usize].stop();
                self.process_lst[last as usize].name = None;
                to_be_remove -= 1;
                last -= 1;
                self.process_lst.pop();
            }
        }
    }

    pub fn remove_all_process(&mut self) {
        self.stop_all_process();
        self.is_active = false;
    }

    pub fn add_nb_process(&mut self, mut to_be_add: i64) {
        while to_be_add > 0 {
            let index = self.process_lst.len() as i64;
            let name = n_name(self.getnameparse().clone(), index);
            let mut log_file = FileLog::from_file(self.parse_doc_yaml.clone(), Some(name.clone()));
            let mut new_proc = self.proc_file_log(name, &index, &mut log_file);

            new_proc.set_args_task(&self.parse_file);
            new_proc.setup_command_umask(&self.parse_file);
            new_proc.stopsignal = self.parse_file.stopsignal;
            new_proc.exp_exit = self.parse_file.exitcodes.clone();
            self.process_lst.push(new_proc);

            to_be_add -= 1;
        }
    }
}
