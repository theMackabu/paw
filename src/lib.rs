mod structs;

use psutil::process::{MemoryInfo, Process};
use std::io::{self, Read};
use std::process::{Command, Stdio};
use std::thread::{sleep, spawn};
use std::time::{Duration, Instant};
use structs::{PawDone, PawInfo, PawProcess, PawResult};

/// Represents a process watcher that monitors the resource usage of a child process.
#[derive(Debug, Clone)]
pub struct Paw {
    command: String,
    arguments: Vec<String>,
    duration: Duration,
}

impl Paw {
    /// Creates a new `Paw` instance with the specified command, arguments, and duration.
    pub fn new<T: AsRef<str>>(command: &str, arguments: &[T], duration: u64) -> Paw {
        Paw {
            command: command.to_string(),
            arguments: arguments.iter().map(|s| s.as_ref().to_string()).collect(),
            duration: Duration::from_millis(duration),
        }
    }

    /// Watches the process and calls the provided callback with the result at regular intervals.
    ///
    /// # Arguments
    ///
    /// * `callback` - A callback function that takes a `PawResult` as its parameter.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `PawDone` indicating the completion status.
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
                    cmd: self.command.clone(),
                    args: self.arguments.clone(),
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
