use std::str::FromStr;
use std::iter::FromIterator;
use std::error::Error;
use std::convert::From;
use std::num::ParseIntError;
use std::fmt::{self, Display, Formatter};

use schedule::{Schedule, Period, Calendar, ScheduleParseError, PeriodParseError};

#[derive(Debug, PartialEq)]
pub enum CrontabEntry {
    EnvVar(EnvVarEntry),
    User(UserCrontabEntry),
    System(SystemCrontabEntry),
    Anacron(AnacrontabEntry)
}

#[derive(Debug, PartialEq)]
pub struct EnvVarEntry(pub String, pub String);

#[derive(Debug, PartialEq)]
pub struct UserCrontabEntry {
    pub sched: Schedule,
    pub cmd: String
}

#[derive(Debug, PartialEq)]
pub struct SystemCrontabEntry {
    pub sched: Schedule,
    pub user: UserInfo,
    pub cmd: String
}

#[derive(Debug, PartialEq)]
pub struct AnacrontabEntry {
    pub period: Period,
    pub delay: u32,
    pub jobid: String,
    pub cmd: String
}

impl From<UserCrontabEntry> for CrontabEntry {
    fn from(entry: UserCrontabEntry) -> CrontabEntry {
        CrontabEntry::User(entry)
    }
}

impl From<SystemCrontabEntry> for CrontabEntry {
    fn from(entry: SystemCrontabEntry) -> CrontabEntry {
        CrontabEntry::System(entry)
    }
}

impl From<AnacrontabEntry> for CrontabEntry {
    fn from(entry: AnacrontabEntry) -> CrontabEntry {
        CrontabEntry::Anacron(entry)
    }
}

impl From<EnvVarEntry> for CrontabEntry {
    fn from(entry: EnvVarEntry) -> CrontabEntry {
        CrontabEntry::EnvVar(entry)
    }
}

impl Display for CrontabEntry {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            CrontabEntry::EnvVar(ref entry) => entry.fmt(f),
            CrontabEntry::Anacron(ref entry) => entry.fmt(f),
            CrontabEntry::User(ref entry) => entry.fmt(f),
            CrontabEntry::System(ref entry) => entry.fmt(f),
        }
    }
}

impl CrontabEntry {
    pub fn period<'a>(&'a self) -> Option<&'a Period> {
        match *self {
            CrontabEntry::Anacron(AnacrontabEntry { ref period, .. }) => Some(period),
            CrontabEntry::User(UserCrontabEntry { sched: Schedule::Period(ref period), .. }) => Some(period),
            CrontabEntry::System(SystemCrontabEntry { sched: Schedule::Period(ref period), .. }) => Some(period),
            _ => None
        }
    }

    pub fn calendar<'a>(&'a self) -> Option<&'a Calendar> {
        match *self {
            CrontabEntry::User(UserCrontabEntry { sched: Schedule::Calendar(ref cal), .. }) => Some(cal),
            CrontabEntry::System(SystemCrontabEntry { sched: Schedule::Calendar(ref cal), .. }) => Some(cal),
            _ => None
        }
    }

    pub fn command<'a>(&'a self) -> Option<&'a str> {
        match *self {
            CrontabEntry::User(UserCrontabEntry { ref cmd, .. }) => Some(&**cmd),
            CrontabEntry::System(SystemCrontabEntry { ref cmd, .. }) => Some(&**cmd),
            CrontabEntry::Anacron(AnacrontabEntry { ref cmd, .. }) => Some(&**cmd),
            _ => None
        }
    }

    pub fn user<'a>(&'a self) -> Option<&'a str> {
        match *self {
            CrontabEntry::System(SystemCrontabEntry { user: UserInfo(ref user, _, _), .. }) => Some(&**user),
            _ => None
        }
    }

    pub fn group<'a>(&'a self) -> Option<&'a str> {
        match *self {
            CrontabEntry::System(SystemCrontabEntry { user: UserInfo(_, Some(ref group), _), .. }) => Some(&**group),
            _ => None
        }
    }
}

impl Display for EnvVarEntry {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}={}", self.0, self.1)
    }
}

impl FromStr for EnvVarEntry {
    type Err = CrontabEntryParseError;
    fn from_str(s: &str) -> Result<EnvVarEntry, CrontabEntryParseError> {
        let spaces = [' ', '\t'];
        let mut splits = s.splitn(2, '=');

        let name = match splits.next() {
            Some(n) => n.trim_right_matches(&spaces[..]),
            None => return Err(CrontabEntryParseError::MissingEnvVarName)
        };

        if name.len() == 0 {
            return Err(CrontabEntryParseError::MissingEnvVarName)
        }

        if name.chars().any(|v| v == ' ' || v == '\t') {
            return Err(CrontabEntryParseError::InvalidEnvVarName)
        }

        let mut value = match splits.next() {
            Some(v) => v.trim_left_matches(&spaces[..]),
            None => return Err(CrontabEntryParseError::MissingEnvVarValue)
        };

        if value.len() > 1 {
            if &value[..1] == "'" || &value[..1] == "\"" && &value[..1] == &value[value.len()-1..] {
                value = &value[1..value.len()-1];
            }
        }

        Ok(EnvVarEntry(name.to_owned(), value.to_owned()))
    }
}

#[derive(Debug, PartialEq)]
// user, group, class
pub struct UserInfo(pub String, pub Option<String>, pub Option<String>);

#[derive(Debug, PartialEq)]
pub struct UserInfoParseError;

impl Display for UserInfo {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        try!(self.0.fmt(f));
        if let Some(ref group) = self.1 {
            try!(f.write_str(":"));
            try!(group.fmt(f));
        }
        if let Some(ref class) = self.2 {
            try!(f.write_str(":"));
            try!(class.fmt(f));
        }
        Ok(())
    }
}

impl FromStr for UserInfo {
    type Err = UserInfoParseError;
    fn from_str(s: &str) -> Result<UserInfo, UserInfoParseError> {
        let mut splits = s.split(':');
        Ok(UserInfo(
            try!(splits.next().ok_or(UserInfoParseError).map(ToOwned::to_owned)),
            splits.next().map(ToOwned::to_owned),
            splits.next().map(ToOwned::to_owned)
        ))
    }
}

impl Error for UserInfoParseError {
    fn description(&self) -> &str {
        "invalid user name"
    }
    fn cause(&self) -> Option<&Error> {
        None
    }
}

impl Display for UserInfoParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("invalid user name")
    }
}

#[derive(Debug, PartialEq)]
pub enum CrontabEntryParseError {
    InvalidSchedule(ScheduleParseError),
    InvalidPeriod(PeriodParseError),
    InvalidUser(UserInfoParseError),
    InvalidDelay(ParseIntError),
    InvalidEnvVarName,
    MissingPeriod,
    MissingDelay,
    MissingJobId,
    MissingEnvVarName,
    MissingEnvVarValue,
}

impl Display for CrontabEntryParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use self::CrontabEntryParseError::*;
        match *self {
            InvalidSchedule(ref e) => write!(f, "invalid schedule: {}", e),
            InvalidPeriod(ref e) => write!(f, "invalid period: {}", e),
            InvalidUser(ref e) => write!(f, "invalid user: {}", e),
            InvalidDelay(ref e) => write!(f, "invalid delay: {}", e),
            _ => f.write_str(self.description()),
        }
    }
}

impl Error for CrontabEntryParseError {
    fn description(&self) -> &str {
        use self::CrontabEntryParseError::*;
        match *self {
            InvalidSchedule(_) => "invalid schedule",
            InvalidPeriod(_) => "invalid period",
            InvalidUser(_) => "invalid user",
            InvalidDelay(_) => "invalid delay",
            InvalidEnvVarName => "invalid environment variable name",
            MissingPeriod => "missing period",
            MissingDelay => "missing delay",
            MissingJobId => "missing jobid",
            MissingEnvVarName => "missing environment variable name",
            MissingEnvVarValue => "missing environment variable value",
        }
    }
    fn cause(&self) -> Option<&Error> {
        use self::CrontabEntryParseError::*;
        match *self {
            InvalidSchedule(ref e) => Some(e),
            InvalidPeriod(ref e) => Some(e),
            InvalidUser(ref e) => Some(e),
            InvalidDelay(ref e) => Some(e),
            _ => None,
        }
    }
}

impl Display for UserCrontabEntry {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} {}", self.sched, self.cmd)
    }
}

impl FromStr for UserCrontabEntry {
    type Err = CrontabEntryParseError;

    fn from_str(s: &str) -> Result<UserCrontabEntry, CrontabEntryParseError> {
        let seps = [' ', '\t'];
        let mut splits = s.split(&seps[..]).filter(|v| *v != "");
        Ok(UserCrontabEntry {
            sched: try!(Schedule::from_iter(&mut splits)),
            cmd: splits.collect::<Vec<&str>>().join(" ")
        })
    }
}

impl Display for SystemCrontabEntry {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} {} {}", self.sched, self.user, self.cmd)
    }
}

impl FromStr for SystemCrontabEntry {
    type Err = CrontabEntryParseError;

    fn from_str(s: &str) -> Result<SystemCrontabEntry, CrontabEntryParseError> {
        let seps = [' ', '\t'];
        let mut splits = s.split(&seps[..]).filter(|v| *v != "");
        Ok(SystemCrontabEntry {
            sched: try!(Schedule::from_iter(&mut splits)),
            user: try!(splits.next().ok_or(UserInfoParseError).and_then(FromStr::from_str)),
            cmd: splits.collect::<Vec<&str>>().join(" ")
        })
    }
}

impl Display for AnacrontabEntry {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "@{} {} {} {}", self.period, self.delay, self.jobid, self.cmd)
    }
}

impl FromStr for AnacrontabEntry {
    type Err = CrontabEntryParseError;

    fn from_str(s: &str) -> Result<AnacrontabEntry, CrontabEntryParseError> {
        let seps = [' ', '\t'];
        let mut splits = s.split(&seps[..]).filter(|v| *v != "");
        Ok(AnacrontabEntry {
            period: try!(splits.next().map(|v| v.parse().map_err(CrontabEntryParseError::InvalidPeriod)).unwrap_or(Err(CrontabEntryParseError::MissingPeriod))),
            delay: try!(splits.next().map(|v| v.parse().map_err(CrontabEntryParseError::InvalidDelay)).unwrap_or(Err(CrontabEntryParseError::MissingDelay))),
            jobid: try!(splits.next().map(ToOwned::to_owned).ok_or(CrontabEntryParseError::MissingJobId)),
            cmd: splits.collect::<Vec<&str>>().join(" ")
        })
    }
}

impl From<ScheduleParseError> for CrontabEntryParseError {
    fn from(e: ScheduleParseError) -> CrontabEntryParseError {
        CrontabEntryParseError::InvalidSchedule(e)
    }
}

impl From<UserInfoParseError> for CrontabEntryParseError {
    fn from(e: UserInfoParseError) -> CrontabEntryParseError {
        CrontabEntryParseError::InvalidUser(e)
    }
}

impl From<ParseIntError> for CrontabEntryParseError {
    fn from(e: ParseIntError) -> CrontabEntryParseError {
        CrontabEntryParseError::InvalidDelay(e)
    }
}
