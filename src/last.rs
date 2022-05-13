use cubeos_error::{Error as CubeOSError, Result as CubeOSResult};

pub trait Last {
    fn set_last_cmd(&self, input: Vec<u8>);
    fn get_last_cmd(&self) -> CubeOSResult<Vec<u8>>;
    fn set_last_err(&self, err: CubeOSError);
    fn get_last_err(&self) -> CubeOSResult<CubeOSError>;
}