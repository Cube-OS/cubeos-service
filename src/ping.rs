use crate::error::Result;

pub trait Ping {
    fn ping(&self) -> Result<()>;
}