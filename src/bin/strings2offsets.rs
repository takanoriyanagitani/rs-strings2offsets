use std::io;
use std::process::ExitCode;

use rs_strings2offsets::Config;

fn io_config() -> impl Fn() -> Config {
    || Config::default()
}

fn io_main() -> impl FnOnce() -> Result<(), io::Error> {
    || {
        let cfg: Config = io_config()();
        cfg.stdin2stdout()
    }
}

fn sub() -> Result<(), io::Error> {
    io_main()()
}

fn main() -> ExitCode {
    sub().map(|_| ExitCode::SUCCESS).unwrap_or_else(|e| {
        eprintln!("{e}");
        ExitCode::FAILURE
    })
}
