use cubeos_error::Result;

pub trait Ping {
    fn ping(&self) -> Result<()>;
}