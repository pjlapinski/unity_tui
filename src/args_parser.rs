use std::{env, path::PathBuf};

pub enum ArgsError {
    TooFew,
}

pub fn parse() -> Result<PathBuf, ArgsError> {
    let mut argv = env::args();
    let _program = argv.next();
    let path = match argv.next() {
        None => return Err(ArgsError::TooFew),
        Some(p) => p,
    };
    Ok(PathBuf::from(path))
}
