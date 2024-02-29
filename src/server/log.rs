use crate::tool::DurationDate;

use super::proc::Proc;

impl Proc {
    pub fn spawn_log(&self) -> String {
        format!(
            "{} INFO spawned: {} with {}",
            self.sys_date.unwrap().format("%d/%m/%Y %H:%M:%S"),
            self.get_name(),
            self.pid.unwrap()
        )
    }

    pub fn running_log(&self) -> String {
        format!(
            "INFO success: {} entered RUNNING state, process has stayed up for {} seconds (starttime)",
            self.get_name(), self.started_time.unwrap().elapsed().durationdate()
        )
        // TODO: random startsecs here, need to be replaced
    }

    pub fn exit_log(&mut self) -> String {
        match &self.get_exit_status() {
            Some(status) => {
                format!(
                    "{} INFO exited: {} (terminated with {status})",
                    self.sys_date.unwrap().format("%d/%m/%Y %H:%M:%S"),
                    self.get_name()
                ) // TODO: miss the expected error code
            }
            None => format!(
                "{} INFO exited: Something went wrong !",
                self.sys_date.unwrap().format("%d/%m/%Y %H:%M:%S")
            ),
        }
    }

    pub fn stopped_log(&mut self) -> String {
        match &self.get_exit_status() {
            Some(status) => {
                format!(
                    "{} INFO stopped: {} (terminated with {status})",
                    self.sys_date.unwrap().format("%d/%m/%Y %H:%M:%S"),
                    self.get_name()
                )
            }
            None => {
                format!(
                    "{} INFO stopped: Something went wrong !",
                    self.sys_date.unwrap().format("%d/%m/%Y %H:%M:%S")
                )
            }
        }
    }

    pub fn stopping_log(&mut self) -> String {
        format!(
            "{} INFO waiting for {} to stop",
            self.sys_date.unwrap().format("%d/%m/%Y %H:%M:%S"),
            self.get_name()
        )
    }

    pub fn gaveup_log(&self) -> String {
        format!(
            "{} INFO gave up :{} entered FATAL state, too many start retries too quickly",
            self.sys_date.unwrap().format("%d/%m/%Y %H:%M:%S"),
            self.name.as_ref().unwrap()
        )
    }

    pub fn spawnerr_log(&self) -> String {
        format!(
            "{:?} INFO spawerr : can't find command '{}' ",
            self.sys_time.unwrap(),
            self.name.as_ref().unwrap()
        )
    }
}