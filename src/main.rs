extern crate dirs;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate walkdir;

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
    let config = get_config()?;
    do_work(&config.log_dir, config.max_size)?;
    Ok(())
}

fn do_work(log_dir: &PathBuf, max_size: u64) -> Result<(), Error> {
    let mut files = HashMap::new();
    let wd = WalkDir::new(log_dir).min_depth(1);
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
        if log.main_size > max_size {
            for num in log.numbers.iter().rev() {
                if num == &10 {
                    ::std::fs::remove_file(log_dir.join(&format!("{}.10.log", name)))?;
                } else if num == &0 {
                    let from = log_dir.join(&format!("{}.log", name));
                    let to = log_dir.join(&format!("{}.1.log", name));
                    ::std::fs::copy(from.clone(), to)?;
                    ::std::fs::write(from, &[])?;
                } else {
                    let from = log_dir.join(&format!("{}.{}.log", name, num));
                    let to = log_dir.join(&format!("{}.{}.log", name, num + 1));
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
    let name = parts.next().ok_or(Error::Other(format!("unable to get file name {}", full_name)))?;
    let next = parts.next().ok_or(Error::Other(format!("{} is an invalid filename", full_name)))?;
    Ok(
        Some(
            if next == "log" {
                (name.to_string(), 0)
            } else {
                let n: u8 = next.parse().map_err(|e| {
                    eprintln!("Error parsing {}", full_name);
                    e
                })?;
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

#[derive(Deserialize)]
struct Config {
    pub log_dir: PathBuf,
    pub max_size: u64,
}

fn get_config() -> Result<Config, Error> {
    let home = home_dir().ok_or(Error::other("Unable to find home dir"))?;
    if let Ok(config_text) = ::std::fs::read_to_string(home.join(".log_maintenance")) {
        let ret = toml::from_str(&config_text)?;
        Ok(ret)
    } else {
        Ok(
            Config {
                log_dir: home.join("logs"),
                max_size: 1024,
            }
        )
    }
}

#[derive(Debug)]
enum Error {
    Other(String),
    WalkDir(walkdir::Error),
    Io(::std::io::Error),
    ParseInt(::std::num::ParseIntError),
    Toml(toml::de::Error),
}

impl ::std::error::Error  for Error {}

impl ::std::fmt::Display  for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Error::Other(ref s) => s.fmt(f),
            Error::WalkDir(ref e) => e.fmt(f),
            Error::Io(ref e) => e.fmt(f),
            Error::ParseInt(ref e) => e.fmt(f),
            Error::Toml(ref e) => e.fmt(f),
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

impl From<toml::de::Error> for Error {
    fn from(other: toml::de::Error) -> Self {
        Error::Toml(other)
    }
}