use super::Device;
use crate::backend::tty::{gpu_time::GpuTime, schedule::Policy};
use std::error::Error;

impl Device {
    pub fn configure_timing(&mut self, policy: Policy) -> Result<(), Box<dyn Error>> {
        if policy != Policy::Immediate {
            let timer = GpuTime::new(&mut self.renderer)?;
            let label = if policy == Policy::Adaptive {
                "adaptive"
            } else {
                "deadline"
            };
            eprintln!(
                "raven: frame pipeline {label}; GPU elapsed timing={}",
                timer.is_enabled()
            );
            self.gpu_time = Some(timer);
        } else {
            eprintln!("raven: frame pipeline immediate; no render-ahead or GPU queries");
        }
        Ok(())
    }

    pub fn destroy_timing(&mut self) {
        if let Some(timer) = &mut self.gpu_time
            && let Err(error) = timer.destroy(&mut self.renderer)
        {
            eprintln!("raven: GPU query cleanup failed; context teardown will release it: {error}");
        }
    }
}
