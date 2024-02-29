use chrono::Local;
use sha1::{Digest, Sha1};
use std::{fs::File, process::Command};
use yaml_rust::Yaml;

use super::{
    conf::Taskmaster,
    proc::Proc,
    tool::{parse_to_string, test_stderr, test_stdout},
    task::Task,
};

#[derive(Debug)]
pub struct FileLog {
    pub redirect_stderr: Option<bool>,
    pub redirect_stdout: Option<bool>,
    pub path_stdout_logfile: Option<String>,
    pub path_stderr_logfile: Option<String>,
    pub file_stdout: Option<File>,
    pub file_stderr: Option<File>,
    pub name: Option<String>,
}

impl Clone for FileLog {
    fn clone(&self) -> Self {
        Self {
            redirect_stderr: self.redirect_stderr.clone(),
            redirect_stdout: self.redirect_stdout.clone(),
            path_stdout_logfile: self.path_stdout_logfile.clone(),
            path_stderr_logfile: self.path_stderr_logfile.clone(),
            file_stdout: clone_option(self.file_stdout.as_ref()),
            file_stderr: clone_option(self.file_stderr.as_ref()),
            name: self.name.clone(),
        }
    }
}

impl FileLog {
    pub fn from_file(task_file: Yaml, task_name: Option<String>) -> Self {
        FileLog {
            path_stdout_logfile: parse_to_string(task_file["stdout"].as_str()),
            path_stderr_logfile: parse_to_string(task_file["stderr"].as_str()),
            redirect_stderr: task_file["redirect_stderr"].as_bool(),
            redirect_stdout: task_file["redirect_stdout"].as_bool(),
            file_stdout: None,
            file_stderr: None,
            name: task_name,
        }
    }

    pub fn create_generic_name(&self, extension: &str) -> String {
        let str: &String = self.name.as_ref().expect("error name constructor task");
        let str_date = format!("{}", Local::now().format("%H:%M:%S"));
        let final_str = str.to_owned() + &str_date + extension;
        let mut hasher = Sha1::new();
        hasher.update(final_str);
        hex::encode(hasher.finalize())
    }

    pub fn create_generic_file(&mut self, extension: &str) -> Option<File> {
        let namefile: String;
        // let file_tmp_stderr;
        let mut generic = self.create_generic_name(extension);
        generic.truncate(5);
        namefile = "./".to_string()
            + &self.name.as_ref().unwrap()
            + &extension.to_string()
            + &"----taskmaster-".to_string()
            + &generic
            + &".log".to_string();
        Some(File::create(namefile).expect("error_file_tmp_stderr"))
    }

    // on fera les redirections plus tard
    pub fn create_generic_files(&mut self) {
        if self.file_stdout.is_none() {
            let stdout = self.create_generic_file("-stdout");
            self.file_stdout = stdout;
        }
        if self.file_stderr.is_none() {
            let stderr = self.create_generic_file("-stderr");
            self.file_stderr = stderr;
        }
    }
}

pub trait Files {
    fn files(&mut self);
    fn redirection(&mut self);
}

impl Files for Proc {
    fn files(&mut self) {
        self.s_file.create_generic_files();
    }

    fn redirection(&mut self) {
        let f_stdout = self
            .s_file
            .file_stdout
            .as_ref()
            .expect("miss file in redirection? ")
            .try_clone();
        let f_stderr = self
            .s_file
            .file_stderr
            .as_ref()
            .expect("miss file in redirection? ")
            .try_clone();
        self.command
            .as_mut()
            .expect("miss command in redirection ? ")
            .stdout(f_stdout.expect("failed clone ref file stdout?"));
        self.command
            .as_mut()
            .expect("miss command in redirection ? ")
            .stderr(f_stderr.expect("failed clone ref file stderr?"));
    }
}

impl Files for Task {
    fn files(&mut self) {
        self.process_lst.iter_mut().for_each(|task| task.files())
    }

    fn redirection(&mut self) {
        self.process_lst
            .iter_mut()
            .for_each(|task| task.redirection())
    }
}

impl Files for Taskmaster {
    fn files(&mut self) {
        self.task_lst.iter_mut().for_each(|f| f.files());
    }

    fn redirection(&mut self) {
        self.task_lst.iter_mut().for_each(|f| f.redirection());
    }
}

impl Task {
    #[allow(unused_assignments)]
    pub fn proc_file_shared_log(&self, i: &i64, filelog: &mut FileLog, n_name: String) -> Proc {
        let mut file_stdout = None;
        let mut file_stderr = None;

        if !test_stdout(self.getboolfile()) && *i == 0 {
            file_stdout = Some(
                File::create(
                    filelog
                        .path_stdout_logfile
                        .clone()
                        .expect("create specify stdout"),
                )
                .expect("result"),
            );
            filelog.file_stdout = file_stdout;
        }
        if !test_stderr(self.getboolfile()) && *i == 0 {
            file_stderr = Some(
                File::create(
                    filelog
                        .path_stderr_logfile
                        .clone()
                        .expect("create specify stdout"),
                )
                .expect("result"),
            );
            filelog.file_stderr = file_stderr;
        }
        filelog.name = Some(n_name.clone());
        Proc::new(
            Command::new(self.getpathref()),
            filelog.clone(),
            Some(n_name),
        )
    }
}

fn clone_option(opt_arg: Option<&File>) -> Option<File> {
    match opt_arg {
        Some(x) => Some(x.try_clone().expect("error clone?")),
        None => None,
    }
}
