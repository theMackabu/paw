use psutil::process::{MemoryInfo, Process};
use std::io::{self, Read};
use std::process::{Command, Stdio};
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
    pub memory_usage: Option<MemoryInfo>,
    pub cpu_usage: f64,
    pub uptime: u128,
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
        let stdout_thread = std::thread::spawn(move || {
            let mut buffer = String::new();
            let mut reader = io::BufReader::new(stdout_handle);
            reader.read_to_string(&mut buffer).unwrap();
            buffer
        });

        loop {
            let mut memory_usage: Option<MemoryInfo> = None;
            if let Ok(process) = Process::new(pid) {
                if let Ok(info) = process.memory_info() {
                    memory_usage = Some(info);
                }
            }

            let elapsed_time = start_time.elapsed().as_millis();
            let result = PawResult {
                info: PawInfo {
                    memory_usage,
                    cpu_usage: 0.0,
                    uptime: elapsed_time,
                },
                process: PawProcess {
                    cmd: &self.command,
                    args: &self.arguments,
                },
            };

            callback(result);
            std::thread::sleep(self.duration);

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
