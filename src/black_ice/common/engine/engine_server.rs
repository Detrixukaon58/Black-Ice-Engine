use std::error::Error;
use std::fmt::{Debug, Display};
use std::sync::Arc;


pub trait ServerData : Display + Debug{
}

#[derive(Debug)]
pub enum EngineError{
    ReadWriteError{message:String, file: String},
    ReceiveError{message: String, data:Arc<dyn ServerData>},
    RenderError{message: String},

}

impl Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for EngineError{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }

    fn provide<'a>(&'a self, request: &mut std::error::Request<'a>) {
        
    }
}

pub trait EngineServer{
    fn init();
    fn tick() -> EngineError;
    fn recieve(&mut self, data: Arc<dyn ServerData>) -> EngineError;
}