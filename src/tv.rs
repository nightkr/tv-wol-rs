use std::result;

pub type Result<T> = result::Result<T, TVError>;

#[derive(Debug)]
pub enum TVError {
    TVControlFailed
}

pub trait TVController {
    fn turn_on_tv(&mut self) -> Result<()>;
    fn turn_off_tv(&mut self) -> Result<()>;
}

pub struct FakeTVController;

impl FakeTVController {
    pub fn new() -> FakeTVController {
        FakeTVController {}
    }
}

impl TVController for FakeTVController {
    fn turn_on_tv(&mut self) -> Result<()> {
        println!("(mocked) Turning on TV");
        Ok(())
    }

    fn turn_off_tv(&mut self) -> Result<()> {
        println!("(mocked) Turning off TV");
        Ok(())
    }
}
