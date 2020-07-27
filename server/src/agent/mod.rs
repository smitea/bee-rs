use bee_core::{Driver, Instance, Result, DataSource, Connection};

mod filesystem;
mod host_basic;
mod host_cpu;
mod host_mem;
mod host_swap;

pub struct AgentDriver;

// impl Driver for AgentDriver {
//     fn name(&self) -> &str {
//         "agent"
//     }
//
//     fn new_datasource(&self, instance: Instance) -> Result<Box<dyn DataSource>> {
//
//     }
// }

// pub fn new_driver(connect: &dyn Connection) -> Result<Box<dyn Driver>> {
// }