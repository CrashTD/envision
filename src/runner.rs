#[derive(PartialEq, Eq, Debug)]
pub enum RunnerStatus {
    Running,
    Stopped(Option<i32>),
}

pub trait Runner {
    fn start(&mut self);
    fn status(&mut self) -> RunnerStatus;
    fn consume_output(&mut self) -> String;
    fn consume_rows(&mut self) -> Vec<String>;
}
