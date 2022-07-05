use cubeos_error::{Error as CubeOSError, Result as CubeOSResult};

pub trait Ping {
    fn ping(&self) -> CubeOSResult<()>;
}