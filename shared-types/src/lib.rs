#![deny(warnings)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;


#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ProcessInfo {
    pub pid: u32,
    pub mem_percent: f32, // TODO: newtype?
    pub cpu_percent: f32,
    pub cmd_line: String,
}

pub type ProcessName = String;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UpdateResp {
    pub process_map: HashMap<ProcessName, Vec<ProcessInfo>>,
}
