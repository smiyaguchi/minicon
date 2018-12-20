use std::io::Error as IoError;
use nix::Error as NixError;
use std::fmt;
use std::fmt::Display;
use failure::{Backtrace, Context, Fail};

#[derive(Fail, Debug)]
pub enum ErrorKind {
    #[fail(display = "IO error")]
    Io,
    #[fail(display = "Nix error")]
    Nix,    
}

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,    
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()    
    }    
    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()    
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)    
    }    
}

impl Error {
    pub fn new(inner: Context<ErrorKind>) -> Error {
        Error { inner }    
    }   
    pub fn kind(&self) -> &ErrorKind {
        self.inner.get_context()    
    } 
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error {
            inner: Context::new(kind),    
        }    
    }    
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error { inner }    
    }    
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Error {
        Error {
            inner: error.context(ErrorKind::Io),        
        }    
    }    
}

impl From<NixError> for Error {
    fn from(error: NixError) -> Error {
        Error {
            inner: error.context(ErrorKind::Nix),    
        }    
    }    
}

