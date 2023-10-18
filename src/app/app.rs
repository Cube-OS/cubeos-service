use gpio::GpioOut;
use gpio::sysfs::SysFsGpioOutput;
use std::process::Command as ProcessCommand;
use crate::error::{Error,Result};

pub struct Power {
    pub io: u8,
    pub pin4: SysFsGpioOutput,
    pub pin2: SysFsGpioOutput,
    pub pin1: SysFsGpioOutput,
}
impl Power {
    pub fn new(io: Option<u8>) -> Option<Self> {
        if io.is_some() {
            Some(Power {
                io: io.unwrap(),
                pin4: gpio::sysfs::SysFsGpioOutput::open(45).unwrap(),
                pin2: gpio::sysfs::SysFsGpioOutput::open(47).unwrap(),
                pin1: gpio::sysfs::SysFsGpioOutput::open(27).unwrap(),
            })
        } else {
            None
        }
    }

    fn initialize_payload(&mut self) -> Result<()> {   
        let pin1_value = self.io & 0b001;
        let pin2_value = (self.io & 0b010) >> 1;
        let pin4_value = (self.io & 0b100) >> 2;
    
        self.pin4.set_value(pin4_value)?;
        self.pin2.set_value(pin2_value)?;
        self.pin1.set_value(pin1_value)?;
        Ok(())
    }
    
    fn shutdown(&mut self) -> Result<()>{
        self.pin4.set_value(0)?;
        self.pin2.set_value(0)?;
        self.pin1.set_value(0)?;
        Ok(())
    }
}

pub struct App<F> {
    app: F,
    service: String,
    power: Option<Power>,
}

impl<F: Copy> App<F>
where 
    F: FnOnce() -> Result<()>,
{
    pub fn new(app: F, io: Option<u8>, service: &str) -> Self {
        App {
            app,
            service: service.to_string(),
            power: Power::new(io),            
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let child = self.init_resources()?;
        (self.app)()?;
        Ok(self.shutdown_resources(child)?)
    }

    fn init_resources(&mut self) -> Result<std::process::Child> {

        // Start Payload via GPIOs
        if self.power.is_some() {
            self.power.as_mut().unwrap().initialize_payload()?;
        }
        let child = self.start_service()?;

        std::thread::sleep(std::time::Duration::from_secs(1));

        Ok(child)
    }

    fn shutdown_resources(&mut self, mut child: std::process::Child) -> Result<()> {
        if self.power.is_some() {
            self.power.as_mut().unwrap().shutdown()?;
        }
        Ok(child.kill()?)
    }
    
    fn start_service(&self) -> Result<std::process::Child> {
        ProcessCommand::new(self.service.as_str())
            .spawn()
            .map_err(|e| Error::from(e))
    }
}