use std::fs;

use super::parse::File;
use yaml_rust::{Yaml, YamlLoader};


pub trait DurationDate {
    fn durationdate(&self) -> String;
}

impl DurationDate for std::time::Duration {
    fn durationdate(&self) -> String {
        let secs = self.as_secs() % 60;
        let minutes = (self.as_secs() / 60) % 60;
        let hours = ((self.as_secs()) / 60) / 60;
        let str_return = format!("{}:{:02}:{:02}", hours, minutes, secs);
        str_return
    }
}

pub fn n_name(name: String, nb_iter: i64) -> String {
    let n_name = name + &"_".to_string() + &nb_iter.to_string();
    n_name
}

pub fn parse_to_string(opt: Option<&str>) -> Option<String> {
    match opt {
        Some(opt) => Some(opt.to_string()),
        None => None,
    }
}

pub fn test_stdout(parse_file: &File) -> bool {
    parse_file.bool_stdout
}

pub fn test_stderr(parse_file: &File) -> bool {
    parse_file.bool_stderr
}

#[allow(dead_code)]
pub fn test_nofile(parse_file: &File) -> bool {
    test_stdout(parse_file) | test_stderr(parse_file)
}

pub fn file_to_yaml(path: &str) -> Yaml {
    let strfile: String = fs::read_to_string(path).unwrap();
    let docs = YamlLoader::load_from_str(&strfile).unwrap();
    docs[0].clone()
}