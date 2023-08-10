use gpio::GpioOut;
use gpio::sysfs::SysFsGpioOutput;
use std::process::Command as ProcessCommand;
use crate::error::{Error,Result};

pub struct App<F> {
    app: F,
    io: u8,
    service: String,
    pin0: SysFsGpioOutput,
    pin1: SysFsGpioOutput,
    pin2: SysFsGpioOutput,
}

impl<F: Copy> App<F>
where 
    F: FnOnce() -> Result<()>,
{
    pub fn new(app: F, io: u8, service: &str) -> Self {
        App {
            app,
            io,
            service: service.to_string(),
            pin0: gpio::sysfs::SysFsGpioOutput::open(45).unwrap(),
            pin1: gpio::sysfs::SysFsGpioOutput::open(47).unwrap(),
            pin2: gpio::sysfs::SysFsGpioOutput::open(27).unwrap(),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let child = self.init_resources()?;
        (self.app)()?;
        Ok(self.shutdown_resources(child)?)
    }

    fn init_resources(&mut self) -> Result<std::process::Child> {

        // Start Payload via GPIOs
        self.initialize_payload(self.io)?;
        let child = self.start_service()?;

        std::thread::sleep(std::time::Duration::from_secs(1));

        Ok(child)
    }

    fn shutdown_resources(&mut self, mut child: std::process::Child) -> Result<()> {
        child.kill()?;
        Ok(self.shutdown_payload()?)
    }

    fn initialize_payload(&mut self, io: u8) -> Result<()> {   
        let pin0_value = io & 0b001;
        let pin1_value = (io & 0b010) >> 1;
        let pin2_value = (io & 0b100) >> 2;
    
        self.pin0.set_value(pin0_value)?;
        self.pin1.set_value(pin1_value)?;
        self.pin2.set_value(pin2_value)?;
        Ok(())
    }
    
    fn shutdown_payload(&mut self) -> Result<()>{
        self.pin0.set_value(0)?;
        self.pin1.set_value(0)?;
        self.pin2.set_value(0)?;
        Ok(())
    }
    
    fn start_service(&self) -> Result<std::process::Child> {
        ProcessCommand::new(self.service.as_str())
            .arg("-c")
            .arg("config.toml")
            .spawn()
            .map_err(|e| Error::from(e))
    }
}