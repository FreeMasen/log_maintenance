extern crate dirs;
extern crate walkdir;
extern crate regex;

use std::{
    collections::HashMap,
    fs::metadata,
    path::{
        PathBuf,
    },
};

use walkdir::{WalkDir};
use dirs::home_dir;

fn main() -> Result<(), Error> {
    let mut log_dir = home_dir().ok_or(Error::other("Unable to find home dir"))?;
    log_dir.push("logs");
    do_work(&log_dir)?;
    Ok(())
}

fn do_work(p: &PathBuf) -> Result<(), Error> {
    let mut files = HashMap::new();
    let wd = WalkDir::new(p).min_depth(1);
    for entry in wd {
        let entry = entry?;
        if let Some((name, number)) = parse_file_name(&entry)? {
            let log = files.entry(name).or_insert(Log::default());
            if number == 0 {
                let meta = metadata(entry.path())?;
                log.main_size = meta.len();
            }
            log.numbers.push(number);
        }
    }
    for (name, mut log) in files.iter_mut() {
        log.numbers.sort();
        if log.main_size > 1024 * 1024 * 5 {
            for num in log.numbers.iter().rev() {
                if num == &10 {
                    ::std::fs::remove_file(p.join(&format!("{}.10.log", name)))?;
                } else if num == &0 {
                    let from = p.join(&format!("{}.log", name));
                    let to = p.join(&format!("{}.1.log", name));
                    ::std::fs::copy(from.clone(), to)?;
                    ::std::fs::write(from, &[])?;
                } else {
                    let from = p.join(&format!("{}.{}.log", name, num));
                    let to = p.join(&format!("{}.{}.log", name, num + 1));
                    ::std::fs::copy(from.clone(), to)?;
                    ::std::fs::write(from, &[])?;
                }
            }
        }
    }
    Ok(())
}

fn parse_file_name(entry: &walkdir::DirEntry) -> Result<Option<(String, u8)>, Error> {
    let full_name = entry.file_name().to_string_lossy();
    if full_name.starts_with(".") {
        return Ok(None)
    }
    let mut parts = full_name.split('.');
    let name = parts.next().ok_or(Error::other("unable to get file name"))?;
    let next = parts.next().ok_or(Error::other("filename is invalid"))?;
    Ok(
        Some(
            if next == "log" {
                (name.to_string(), 0)
            } else {
                let n: u8 = next.parse()?;
                (name.to_string(), n)
            }
        )
    )
}


struct Log {
    numbers: Vec<u8>,
    main_size: u64,
}

impl ::std::default::Default for Log {
    fn default() -> Self {
        Log {
            numbers: vec![],
            main_size: 0,
        }
    }
}

#[derive(Debug)]
enum Error {
    Other(String),
    WalkDir(walkdir::Error),
    Io(::std::io::Error),
    ParseInt(::std::num::ParseIntError),
    Regex(regex::Error),
}

impl ::std::error::Error  for Error {}

impl ::std::fmt::Display  for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Error::Other(ref s) => s.fmt(f),
            Error::WalkDir(ref e) => e.fmt(f),
            Error::Io(ref e) => e.fmt(f),
            Error::ParseInt(ref e) => e.fmt(f),
            Error::Regex(ref e) => e.fmt(f),
        }
    }
}

impl Error {
    pub fn other(s: &str) -> Self {
        Error::Other(s.into())
    }
}

impl From<walkdir::Error> for Error {
    fn from(other: walkdir::Error) -> Self {
        Error::WalkDir(other)
    }
}

impl From<::std::io::Error> for Error {
    fn from(other: ::std::io::Error) -> Self {
        Error::Io(other)
    }
}

impl From<::std::num::ParseIntError> for Error {
    fn from(other: ::std::num::ParseIntError) -> Self {
        Error::ParseInt(other)
    }
}

impl From<regex::Error> for Error {
    fn from(other: regex::Error) -> Self {
        Error::Regex(other)
    }
}