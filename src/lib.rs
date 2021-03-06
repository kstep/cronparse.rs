use std::ops::{Add, Deref};
use std::fs::File;
use std::io::{self, Lines, BufReader, BufRead};
use std::iter::{Iterator, Enumerate};
use std::error::Error;
use std::convert::{AsRef, From};
use std::path::Path;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

pub trait Limited: Add<u8, Output=Self> + Ord + Copy {
    fn min_value() -> Self;
    fn max_value() -> Self;
}

pub mod schedule;
pub mod interval;
pub mod crontab;

pub struct CrontabFile<T> {
    lines: Enumerate<Lines<BufReader<File>>>,
    _marker: std::marker::PhantomData<T>
}

impl<T> CrontabFile<T> {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<CrontabFile<T>> {
        File::open(path).map(CrontabFile::from_file)
    }

    pub fn from_file(file: File) -> CrontabFile<T> {
        CrontabFile {
            lines: BufReader::new(file).lines().enumerate(),
            _marker: std::marker::PhantomData
        }
    }
}

#[derive(Debug)]
pub enum CrontabFileErrorKind {
    Io(io::Error),
    Parse(crontab::CrontabEntryParseError)
}

impl Display for CrontabFileErrorKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            CrontabFileErrorKind::Io(ref e) => e.fmt(f),
            CrontabFileErrorKind::Parse(ref e) => e.fmt(f)
        }
    }
}

#[derive(Debug)]
pub struct CrontabFileError {
    pub lineno: usize,
    pub line: Option<String>,
    pub kind: CrontabFileErrorKind
}

impl From<io::Error> for CrontabFileError {
    fn from(err: io::Error) -> CrontabFileError {
        CrontabFileError {
            lineno: 0,
            line: None,
            kind: CrontabFileErrorKind::Io(err)
        }
    }
}

impl From<crontab::CrontabEntryParseError> for CrontabFileError {
    fn from(err: crontab::CrontabEntryParseError) -> CrontabFileError {
        CrontabFileError {
            lineno: 0,
            line: None,
            kind: CrontabFileErrorKind::Parse(err)
        }
    }
}

impl Error for CrontabFileError {
    fn description(&self) -> &str {
        "error parsing crontab"
    }

    fn cause(&self) -> Option<&Error> {
        match self.kind {
            CrontabFileErrorKind::Parse(ref e) => Some(e),
            CrontabFileErrorKind::Io(ref e) => Some(e)
        }
    }
}

impl Display for CrontabFileError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "error parsing crontab at line {} ({:?}): {}", self.lineno, self.line.as_ref().map(Deref::deref).unwrap_or("<???>"), self.kind)
    }
}

impl<T> Iterator for CrontabFile<T>
    where T: FromStr,
          crontab::CrontabEntry: From<T>,
          CrontabFileError: From<<T as FromStr>::Err>
{
    type Item = Result<crontab::CrontabEntry, CrontabFileError>;
    fn next(&mut self) -> Option<Result<crontab::CrontabEntry, CrontabFileError>> {
        loop {
            match self.lines.next() {
                Some((lineno, Ok(line))) => {
                    if line.len() == 0 || line.starts_with("#") || line.chars().all(|c| c == ' ' || c == '\t') {
                        continue;
                    }

                    return Some(match line.parse::<crontab::EnvVarEntry>() {
                        Ok(envvar) => Ok(crontab::CrontabEntry::EnvVar(envvar)),
                        _ => line.parse::<T>().map_err(|e| {
                            let mut err: CrontabFileError = From::from(e);
                            err.lineno = lineno + 1;
                            err.line = Some(line.to_owned());
                            err
                        }).map(From::from)
                    });
                },
                Some((lineno, Err(e))) => {
                    let mut err: CrontabFileError = From::from(e);
                    err.lineno = lineno + 1;
                    return Some(Err(err));
                },
                None => return None
            }
        }
    }
}

