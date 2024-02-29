use core::fmt;
use nix::unistd::{getpid, Pid};
use yaml_rust::Yaml;

use super::proc::Proc;
use super::task::Task;
use super::tool::file_to_yaml;
use share_structures::{CallOn, Prog, Response};

#[derive(Debug, Clone)]
pub struct Config {
    pub path: String,
    pub yaml: Yaml,
}

impl Config {
    pub fn set(path: &str) -> Config {
        Self {
            path: path.to_string(),
            yaml: file_to_yaml(path),
        }
    }

    pub fn reload(&mut self) {
        self.yaml = file_to_yaml(&self.path);
    }
}

pub struct Taskmaster {
    pub ppid: Pid,
    pub config: Config,
    pub task_lst: Vec<Task>,
    pub nprocs: i64,
    //Bonus :file pid
    //Bonus: file log
}

impl fmt::Debug for Taskmaster {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Taskmaster")
            .field("pid", &self.ppid)
            .field("nprocs", &self.nprocs)
            .field("tasks", &self.task_lst)
            .finish()
    }
}

impl Taskmaster {
    pub fn new(path: &str) -> Self {
        let conf = Config::set(path);

        Taskmaster {
            ppid: getpid(),
            config: conf.clone(),
            task_lst: Self::set_task_lst(&conf.yaml),
            nprocs: 0,
        }
    }

    pub fn update_config(&mut self) {
        self.config.reload();
    }

    fn set_task_lst(config: &Yaml) -> Vec<Task> {
        // generate empty task_lst
        let mut vec_task: Vec<Task> = Self::init_task_lst(config);

        vec_task.iter_mut().for_each(|task| {
            task.parse_file.init_args();
        });
        vec_task
    }

    fn init_task_lst(config: &Yaml) -> Vec<Task> {
        let mut vec_task: Vec<Task> = Vec::new();
        let mut i: usize = 0;

        loop {
            if config["programs"][i].is_badvalue() {
                break;
            }
            vec_task.push(Task::new(&config["programs"][i]));
            i += 1;
        }
        vec_task
    }

    pub fn set_all_command(&mut self) {
        let mut numprocs = 0;
        let mut task_names: Vec<String> = vec![];

        for task in &mut self.task_lst {
            let name: String = task.getnameparse().clone();
            if !task_names.contains(&name) {
                task_names.push(name.clone());
                numprocs += task.set_command(name)
            }
        }
        self.nprocs = numprocs;
    }

    // ______________________ Actions _________________________
    pub fn start_all_task(&mut self) {
        for i in &mut self.task_lst {
            if i.autostart() == true {
                i.start_all_proc();
            } else {
                i.stop_all_process();
            }
        }
    }

    pub fn start(&mut self, name: String) -> Response {
        let proc = self.get_proc_by_name(name.clone()).unwrap();
        let res = proc.start();

        Response {
            cmd: CallOn::Start(name),
            content: self.get_proc_list(),
            success: res,
        }
    }

    pub fn status(&mut self) -> Response {
        Response {
            cmd: CallOn::Status,
            content: self.get_proc_list(),
            success: Ok(format!("")),
        }
    }

    pub fn stop(&mut self, name: String) -> Response {
        let proc = self.get_proc_by_name(name.clone()).unwrap();
        let res = proc.stop();

        Response {
            cmd: CallOn::Stop(name),
            content: self.get_proc_list(),
            success: res,
        }
    }

    pub fn restart(&mut self, name: String) -> Response {
        let proc = self.get_proc_by_name(name.clone()).unwrap();
        let res = proc.restart();

        Response {
            cmd: CallOn::Restart(name),
            success: res,
            content: self.get_proc_list(),
        }
    }

    pub fn reload(&mut self) -> Response {
        let new_conf = file_to_yaml(&self.config.path);
        let res: Result<String, String>;

        if new_conf == self.config.yaml {
            res = Ok(format!("Reload file with success: 0 change"));
        } else {
            let mut new = Taskmaster::new(&self.config.path);

            new.set_all_command();
            self.update_config();
            self.remove_tasks(&mut new);
            self.add_tasks(new);
            self.start_all_task();
            println!("{:#?}", self);
            res = Ok("Reload with success".to_string());
        }

        Response {
            cmd: CallOn::Reload,
            content: self.get_proc_list(),
            success: res,
        }
    }

    // ____________________ Setter ____________________________
    pub fn add_tasks(&mut self, new: Taskmaster) {
        for new_t in new.task_lst {
            match self.get_task_by_name(&new_t.parse_file.name) {
                Ok(curr) => {
                    // set new config to task
                    curr.parse_doc_yaml = new_t.parse_doc_yaml.clone();
                    curr.parse_file = new_t.parse_file.clone();
                    // Control numprocs add if it needed
                    curr.add_nb_process(
                        new_t.process_lst.len() as i64 - curr.process_lst.len() as i64,
                    );
                }
                Err(_) => self.task_lst.push(new_t),
            };
        }
        self.nprocs = new.nprocs;
    }

    #[allow(unused_must_use)]
    pub fn remove_tasks(&mut self, new: &mut Taskmaster) {
        for task in &mut self.task_lst {
            // New Config Task exists in current ?
            match new.get_task_by_name(&task.parse_file.name) {
                Ok(new_task) => {
                    // set new config to task
                    task.parse_doc_yaml = new_task.parse_doc_yaml.clone();
                    task.parse_file = new_task.parse_file.clone();
                    task.process_lst.iter_mut().for_each(|proc| {
                            proc.stopsignal = new_task.parse_file.stopsignal
                        }
                    );
                    // control numprocs, rm proc if needed
                    task.remove_nb_process(
                        task.process_lst.len() as i64 - new_task.process_lst.len() as i64,
                    );
                }
                Err(_) => task.remove_all_process(),
            };
        }
    }

    // _____________________ Getter ___________________________
    pub fn get_proc_list(&mut self) -> Vec<Prog> {
        let mut proc_list: Vec<Prog> = vec![];

        for task in &self.task_lst {
            for proc in &task.process_lst {
                proc_list.push(Prog {
                    proc_name: proc.get_name(),
                    status: proc.get_state(),
                    info: proc.get_current_description(),
                });
            }
        }
        proc_list
    }

    pub fn get_proc_by_name(&mut self, to_be_found: String) -> Result<&mut Proc, String> {
        for task in &mut self.task_lst {
            for proc in &mut task.process_lst {
                let name_matching = proc.name.clone().map(|name| name == to_be_found).unwrap();
                if name_matching == true {
                    return Ok(proc);
                }
            }
        }
        return Err(format!("Process not found"));
    }

    pub fn get_task_by_name(&mut self, to_be_found: &Option<String>) -> Result<&mut Task, String> {
        for task in &mut self.task_lst {
            match to_be_found.clone() {
                Some(name_to_find) => {
                    let name_matching = task
                        .parse_file
                        .name
                        .as_ref()
                        .map(|name| name == &name_to_find)
                        .unwrap();
                    if name_matching == true {
                        return Ok(task);
                    }
                }
                None => return Err(format!("Task has no name")),
            }
        }
        return Err(format!("Task not found"));
    }

    pub fn name_exist(&self, to_find: String) -> bool {
        for task in &self.task_lst {
            let exist = task
                .parse_file
                .name
                .as_ref()
                .map(|name| name == &to_find)
                .unwrap_or(false);
            if exist {
                return exist;
            }
        }
        false
    }
}
