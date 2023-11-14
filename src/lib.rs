use psutil::process::{MemoryInfo, Process};
use std::io::{self, Read};
use std::process::{Command, Stdio};
use std::thread::{sleep, spawn};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Paw {
    command: String,
    arguments: Vec<String>,
    duration: Duration,
}

#[derive(Debug, Clone)]
pub struct PawResult<'a> {
    pub info: PawInfo,
    pub process: PawProcess<'a>,
}

#[derive(Debug, Clone)]
pub struct PawProcess<'a> {
    pub cmd: &'a String,
    pub args: &'a Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PawInfo {
    pub uptime: u128,
    pub memory_usage: Option<MemoryInfo>,
    pub cpu_percent: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct PawDone {
    pub stdout: String,
    pub code: Option<i32>,
}

impl Paw {
    pub fn new<T: AsRef<str>>(command: &str, arguments: &[T], duration: u64) -> Paw {
        Paw {
            command: command.to_string(),
            arguments: arguments.iter().map(|s| s.as_ref().to_string()).collect(),
            duration: Duration::from_millis(duration),
        }
    }

    pub fn watch<F: Fn(PawResult) + Send + 'static>(&self, callback: F) -> Result<PawDone, Box<dyn std::error::Error>> {
        let mut child = Command::new(&self.command).args(&self.arguments).stdout(Stdio::piped()).spawn()?;

        let done: PawDone;
        let pid = child.id();
        let start_time = Instant::now();

        let stdout_handle = child.stdout.take().unwrap();
        let stdout_thread = spawn(move || {
            let mut buffer = String::new();
            let mut reader = io::BufReader::new(stdout_handle);
            reader.read_to_string(&mut buffer).unwrap();
            buffer
        });

        loop {
            let uptime = start_time.elapsed().as_millis();
            let mut memory_usage: Option<MemoryInfo> = None;
            let mut cpu_percent: Option<f32> = None;

            if let Ok(mut process) = Process::new(pid) {
                memory_usage = process.memory_info().ok();
                cpu_percent = process.cpu_percent().ok();
            }

            let result = PawResult {
                info: PawInfo { memory_usage, cpu_percent, uptime },
                process: PawProcess {
                    cmd: &self.command,
                    args: &self.arguments,
                },
            };

            callback(result);
            sleep(self.duration);

            if let Some(status) = child.try_wait()? {
                done = PawDone {
                    stdout: stdout_thread.join().unwrap(),
                    code: status.code(),
                };

                break;
            }
        }

        Ok(done)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn watch() {
        let paw = Paw::new("node", &["tests/test.js"], 500);
        let callback = move |result: PawResult| {
            println!("{:?}", result);
        };

        match paw.watch(callback) {
            Ok(result) => println!("{:?}", result),
            Err(error) => println!("{error}"),
        }
    }
}
