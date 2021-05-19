use std::fmt;

#[derive(Debug)]
pub struct PingError;


impl fmt::Display for PingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ping error!")
    }
}

#[derive(Debug)]
pub struct JoinError;


impl fmt::Display for JoinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "join error!")
    }
}


