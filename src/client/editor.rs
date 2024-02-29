use share_structures::{CallOn, Request, Response};
use std::io::{stdin, stdout, Write};
use termion::input::TermRead;
use termion::{event::Key, raw::IntoRawMode};

use super::history::History;

pub struct Editor {
    quit: bool,
    newline: bool,
    clear: bool,
    proc_lst: Vec<String>,
}

impl Editor {
    fn init() -> Self {
        Self {
            quit: false,
            newline: true,
            clear: false,
            proc_lst: vec![],
        }
    }

    fn init_client(&mut self) {
        let initial_status = Request::send(&"status".to_string(), vec![]);

        match initial_status {
            Ok(response) => {
                let status = self.get_status(response);
                print!("{}\n\r", status);
            }
            Err(e) => {
                print!("{}\n\r", e);
            }
        }
        stdout().flush().unwrap();
    }

    pub fn read(mut history: History) {
        let mut edit = Self::init();
        let mut line = String::new();
        let mut stdout = stdout().lock().into_raw_mode().unwrap();

        edit.init_client();

        loop {
            edit.clear_line(&mut line);
            edit.write_prompt();
            print!("{}", line);
            stdout.flush().unwrap();
            if edit.newline {
                line.clear();
                edit.newline = false;
            }
            if edit.clear {
                edit.clear_screen();
                edit.clear = false;
            }

            edit.key_manager(&mut history, &mut line);

            if edit.quit {
                println!("\r");
                break;
            }
        }
    }

    pub fn key_manager(&mut self, history: &mut History, line: &mut String) {
        if let Ok(c) = stdin().keys().next().unwrap() {
            match c {
                Key::Char('q') | Key::Ctrl('c') => self.quit = true,
                Key::Backspace => {
                    line.pop();
                }
                Key::Up => {
                    let previous_cmd = history.get(history.index - 1);
                    self.clear_line(line);
                    line.clear();
                    line.push_str(&previous_cmd);
                }
                Key::Down => {
                    let next_cmd = history.get(history.index + 1);
                    self.clear_line(line);
                    line.clear();
                    line.push_str(&next_cmd);
                }
                Key::Ctrl('l') => self.clear = true,
                Key::Char('\n') => {
                    self.newline = true;
                    history.add(line.clone());
                    match Request::send(&line, self.proc_lst.clone()) {
                        Ok(res) => match res.cmd {
                            CallOn::Status => line.push_str(self.get_status(res).as_str()),
                            _ => match res.success {
                                Ok(success) => line.push_str(&format!("\n\r{}", success)),
                                Err(failed) => line.push_str(&format!("\n\r{}", failed)),
                            }
                        },
                        Err(e) => line.push_str(format!("\n\r\t{}", e).as_str()),
                    };
                    line.push_str("\n\rtaskctl > ");
                }
                Key::Char(c) => line.push(c),
                _ => {}
            }
        }
    }

    pub fn get_status(&mut self, response: Response) -> String {
        let mut line = String::new();

        for proc in response.content {
            self.proc_lst.push(proc.proc_name.clone());
            line.push_str(
                format!(
                    "\n\r{}\t\t\t{:?}\t\t{}",
                    proc.proc_name,
                    proc.status,
                    proc.info
                ).as_str(),
            );
        }
        line
    }

    fn clear_screen(&self) {
        print!(
            "{}{}taskctl > ",
            termion::cursor::Goto(1, 1),
            termion::clear::All
        );
        stdout().flush().unwrap();
    }

    fn clear_line(&self, _line: &mut String) {
        print!("{}\r", termion::clear::CurrentLine);
        stdout().flush().unwrap();
    }

    fn write_prompt(&self) {
        print!("taskctl > ");
        stdout().flush().unwrap();
    }

    #[allow(dead_code)]
    fn debug(&self, history: &mut History, line: &mut String) {
        println!("----------------------------");
        print!("HISTO [{:?}]", history.histo);
        print!("Current line : [{}]", line);
        print!("----------------------------\n");
        stdout().flush().unwrap();
    }
}
