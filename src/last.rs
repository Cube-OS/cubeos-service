use cubeos_error::{Error, Result};

pub trait Last {
    fn set_last_cmd(&self, input: Vec<u8>);
    fn get_last_cmd(&self) -> Result<Vec<u8>>;
    fn set_last_err(&self, err: Error);
    fn get_last_err(&self) -> Result<Error>;
}