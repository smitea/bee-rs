use crate::{Error, Session};
use heim::units::{time, Time};
use std::time::Duration;
use std::{
    io::Result,
    process::{Command, Output},
};

mod cpu_usage;
mod filesystem_usage;
mod memory_usage;
mod os_basic;
mod os_info;
mod swap_usage;

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

    if output.status.success() {
        String::from_utf8(output.stdout).or_else(|err| {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("format err - {:?}", err),
            ))
        })
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("exit code: {:?}", output.status),
        ))
    }
}

pub fn init_datasource(sess: &mut dyn Session) -> std::result::Result<(), Error> {
    sess.register_source(Box::new(cpu_usage::CPUUsage))?;
    sess.register_source(Box::new(filesystem_usage::FileSystemUsage))?;
    sess.register_source(Box::new(memory_usage::MemoryUsage))?;
    sess.register_source(Box::new(os_basic::OSBasic))?;
    sess.register_source(Box::new(os_info::OSInfo))?;
    sess.register_source(Box::new(swap_usage::SWAPUsage))?;
    Ok(())
}
