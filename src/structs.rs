use psutil::process::MemoryInfo;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PawResult {
    pub info: PawInfo,
    pub process: PawProcess,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PawProcess {
    pub cmd: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PawInfo {
    pub uptime: u128,
    pub memory_usage: Option<MemoryInfo>,
    pub cpu_percent: Option<f32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PawDone {
    pub stdout: String,
    pub code: Option<i32>,
}
