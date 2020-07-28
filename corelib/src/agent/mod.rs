use crate::{Connection, Error,Result, Instance};
use heim::units::{time, Time};
use std::time::Duration;
use std::{
    process::{Command, Output},
};

mod filesystem;
mod host_basic;
mod host_cpu;
mod host_mem;
mod host_swap;
mod host_info;

impl From<heim::Error> for Error {
    fn from(err: heim::Error) -> Self {
        return Error::internal(0x00, format!("{}", err));
    }
}

fn format(t: Time) -> u64 {
    let duration = Duration::from_secs_f64(t.get::<time::second>());
    duration.as_secs()
}

fn run_command(cmd: &str) -> Result<String> {
    let output: Output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(&["/C", &cmd]).output()?
    } else {
        Command::new("sh").arg("-c").arg(&cmd).output()?
    };

    let line = if output.status.success() {
        String::from_utf8(output.stdout).or_else(|err| {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("format err - {:?}", err),
            ))
        })?
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("exit code: {:?}", output.status),
        ))?
    };

    Ok(line)
}

pub fn register_ds<T: Connection>(instance: &Instance,connection: &T) -> Result<()>{
    Ok(())
}