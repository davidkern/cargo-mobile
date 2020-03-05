use ginit_core::exports::{
    into_result::{command::CommandError, IntoResult as _},
    once_cell_regex::regex,
};
use std::{
    fmt::{self, Display},
    process::Command,
    str,
};

#[derive(Debug)]
pub enum Error {
    SystemProfilerFailed(CommandError),
    OutputInvalidUtf8(str::Utf8Error),
    VersionNotMatched {
        data: String,
    },
    MajorVersionInvalid {
        major: String,
        cause: std::num::ParseIntError,
    },
    MinorVersionInvalid {
        minor: String,
        cause: std::num::ParseIntError,
    },
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::SystemProfilerFailed(err) => write!(f, "`system_profiler` call failed: {}", err),
            Error::OutputInvalidUtf8(err) => write!(
                f,
                "`system_profiler` output contained invalid UTF-8: {}",
                err
            ),
            Error::VersionNotMatched { data } => write!(
                f,
                "No version number was found within the `SPDeveloperToolsDataType` data: {:?}",
                data
            ),
            Error::MajorVersionInvalid { major, cause } => write!(
                f,
                "The major version {:?} wasn't a valid number: {}",
                major, cause
            ),
            Error::MinorVersionInvalid { minor, cause } => write!(
                f,
                "The minor version {:?} wasn't a valid number: {}",
                minor, cause
            ),
        }
    }
}

// There's a bunch more info available, but the version is all we need for now.
#[derive(Debug)]
pub struct DeveloperTools {
    pub version: (u32, u32),
}

impl DeveloperTools {
    pub fn new() -> Result<Self, Error> {
        let version_re = regex!(r"\bVersion: (?P<major>\d+)\.(?P<minor>\d+)\b");
        // The `-xml` flag can be used to get this info in plist format, but
        // there don't seem to be any high quality plist crates, and parsing
        // XML sucks, we'll be lazy for now.
        let bytes = Command::new("system_profiler")
            .arg("SPDeveloperToolsDataType")
            .output()
            .into_result()
            .map_err(Error::SystemProfilerFailed)
            .map(|out| out.stdout)?;
        let text = str::from_utf8(&bytes).map_err(Error::OutputInvalidUtf8)?;
        let caps = version_re
            .captures(text)
            .ok_or_else(|| Error::VersionNotMatched {
                data: text.to_owned(),
            })?;
        let major = {
            let raw = &caps["major"];
            raw.parse::<u32>()
                .map_err(|cause| Error::MajorVersionInvalid {
                    major: raw.to_owned(),
                    cause,
                })?
        };
        let minor = {
            let raw = &caps["minor"];
            raw.parse::<u32>()
                .map_err(|cause| Error::MinorVersionInvalid {
                    minor: raw.to_owned(),
                    cause,
                })?
        };
        Ok(Self {
            version: (major, minor),
        })
    }
}
