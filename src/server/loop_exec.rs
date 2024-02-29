use std::time::{Instant, Duration};

use share_structures::State;
use super::tool::DurationDate;
use super::parse::File;
use super::proc::Proc;
use super::task::Task;
use super::conf::Taskmaster;


#[derive(Debug, PartialEq, Clone)]
pub enum Autorestart {
    Never,
    Always,
    OnError,
}

pub trait LoopRestart{
    fn start_loop(& mut self);
}

pub fn autostart(task: & mut Task) -> bool{
    task.autostart()
}

impl LoopRestart for Taskmaster {
    fn start_loop(& mut self) {
        self.task_lst
            .iter_mut()
            .filter(|x|x.getautostart() == true)
            .for_each(|x| x.start_loop());
    }
}

impl LoopRestart for Task {
    fn start_loop(& mut self) {
        self.process_lst
            .iter_mut()
            .for_each(|x| x.start_loop())
    }
}

impl LoopRestart for Proc {
    fn start_loop(&mut self) {
        let ret = self.command.as_mut().expect("working at this point ? (Command)").spawn().expect("failed to execute");
        //need a protection w/ the status of the process ...
        self.child.get_or_insert(ret);
        self.nbr_restart += 1;
        self.started_time.get_or_insert(Instant::now());
        self.pid = Some(self.child.as_ref().expect("bah").id());
        self.description = format!("pid {} uptime: {}", self.pid.unwrap(), self.started_time.unwrap().elapsed().durationdate());
        self.change_state(State::STARTING);
    }
}

pub fn exec_loop(vec_process: &mut Vec<Proc>, ref_file: &File) {
    let mut ref_proc: &Proc;

    for i in vec_process {
        ref_proc = i;
        match i.loop_test_file_config //losange
        {
            Some(test_config) => {
                if test_config(&ref_file, ref_proc) == true
                {
                    match i.loop_action_true {
                        Some(x) => x(i),
                        None => (),
                    }
                }
                else {
                    match i.loop_action_false{
                        Some(x) => x(i),
                        None => (),
                    }
                } 
            }
            None => {()},
        }
    }
}

// pub fn exec_proc(proc: &mut Proc, ref_file: &ParseFile) {
//     match proc.loop_test_file_config //losange
//     {
//         Some(test_config) => {
//             if test_config(&ref_file, proc) == true
//             {
//                 match proc.loop_action_true {
//                     Some(x) => x(proc),
//                     None => (),
//                 }
//             }
//             else {
//                 match proc.loop_action_false{
//                     Some(x) => x(proc),
//                     None => (),
//                 }
//             } 
//         }
//         None => {()},
//     }
// }

pub fn test_autostart(fileconf: &File, _ref_proc: &Proc) -> bool {
    fileconf.autostart.unwrap()
}

pub fn test_autorestart_nb(fileconf: &File, ref_proc: &Proc) -> bool {
    let nbr_start_loop = ref_proc.nbr_restart;
    let nbr_restart_expected = fileconf.startretries.unwrap();

    match nbr_start_loop < nbr_restart_expected
    {
        true => true,
        false => false
    }
}

pub fn test_autorestart(fileconf: &File, ref_proc: &Proc) -> bool {
    let autorestart = fileconf.autorestart.as_ref().unwrap();
    match autorestart {
        Autorestart::Never => false,
        Autorestart::Always => test_autorestart_nb(fileconf, ref_proc),
        Autorestart::OnError => {
            match fileconf.exitcodes.contains(&ref_proc.exit_error.unwrap_or_default())
            {
                true => false,
                false => test_autorestart_nb(fileconf, ref_proc),
            }
        }
    }
}

pub fn test_time_starting(fileconf: &File, ref_proc: &Proc) -> bool {
    let startsecs_as_duration = Duration::new(fileconf.starttime.unwrap().try_into().unwrap(), 0);
    match ref_proc.started_time.unwrap().elapsed() > startsecs_as_duration
    {
        true => true,
        false => false
    }
}

pub fn always_true(_fileconf: &File, _ref_proc: &Proc) -> bool {
    true
}

pub fn always_false(_fileconf: &File, _ref_proc: &Proc) -> bool {
    false
}

impl Taskmaster {
    pub fn update_all_task_method(& mut self) {
        for i in & mut self.task_lst{
            for j in & mut i.process_lst{
                j.update_methods()
            }
        }
    }

    pub fn all_exec_loop(& mut self) {
        for i in & mut self.task_lst
        {
            i.call_exec_loop();
        }
    }

    pub fn update_and_exec(& mut self) {
        self.update_all_task_method();
        self.all_exec_loop();
    }

    pub fn all_task_stable(& mut self) -> bool {
        let mut all_bool = true;
        for i in & mut self.task_lst {
            for j in & mut i.process_lst {
                match j.state
                {
                    State::STOPPED |  State::RUNNING | State::FATAL => {
                        all_bool &= true;
                    },
                    _ => {
                        all_bool &= false;
                    },
                }
            }
        }
        all_bool
    }
}
