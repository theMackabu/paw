## Paw

Paw is a Rust library for monitoring and controlling child processes. It provides a simple API for spawning processes, reading their output, and getting their memory and CPU usage.

### Usage

To use Paw, first create a new instance of the `Paw` struct. You can specify the command to run, the arguments to pass to the command, and the duration to monitor the process for.

Once you have a `Paw` instance, you can call the `watch()` method to start monitoring the process. The `watch()` method takes a callback function as an argument. The callback function will be called periodically with the current state of the process.

The `PawResult` struct contains the following information about the process:

- `info`: Information about the process, such as its memory usage, CPU usage, and uptime.
- `process`: Information about the process command, such as the command name and arguments.

The `PawDone` struct contains the following information about the process:

- `stdout`: The standard output of the process.
- `code`: The exit code of the process.

### Example

The following example shows how to use Paw to monitor a Node.js process:

```rust
use paw::{Paw, PawResult};

let paw = Paw::new("node", &["tests/test.js"], 500);
let callback = move |result: PawResult| {
  println!("{:?}", result);
};

match paw.watch(callback) {
  Ok(result) => println!("{:?}", result),
  Err(error) => println!("{error}"),
}
```

### Running the tests

To run the tests, simply run the following command:

```
cargo test
```

### Contributing

If you would like to contribute to Paw, please feel free to open an issue or pull request on GitHub.
