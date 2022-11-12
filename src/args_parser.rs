use std::{env, path::PathBuf};

pub enum ArgsError {
    TooFew,
    NotDir,
}

pub fn parse() -> Result<PathBuf, ArgsError> {
    let mut argv = env::args();
    let _program = argv.next();
    let path = match argv.next() {
        None => return Err(ArgsError::TooFew),
        Some(p) => p,
    };
    let path = PathBuf::from(path);
    if !path.is_dir() {
        Err(ArgsError::NotDir)
    } else {
        Ok(path)
    }
}
