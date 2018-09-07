// Copyright (C) 2014-2015 Mickaël Salaün
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, version 3 of the License.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use std::error::Error;
use std::fmt;
use std::io;
use std::num::ParseIntError;

#[derive(Debug)]
pub struct ParseError {
    desc: String,
    // TODO: cause: Option<&'a (Error + 'a)>,
}

impl ParseError {
    pub fn new(detail: String) -> ParseError {
        ParseError {
            desc: format!("Mount parsing: {}", detail),
        }
    }
}

impl Error for ParseError {
    fn description(&self) -> &str {
        self.desc.as_ref()
    }
}

impl From<io::Error> for ParseError {
    fn from(err: io::Error) -> ParseError {
        ParseError::new(format!("Failed to read the mounts file: {}", err))
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        write!(out, "{}", self.description())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum LineError {
    MissingSpec,
    MissingFile,
    InvalidFilePath(String),
    InvalidFile(String),
    MissingVfstype,
    MissingMntops,
    MissingFreq,
    InvalidFreq(String),
    MissingPassno,
    InvalidPassno(String),

    MissingMountId,
    MissingParentId,
    MissingDevId,
    InvalidDevId(String),
    MissingRoot,
    MissingMountPoint,
    InvalidMountPoint(String),
    MissingMountOpts,
    InvalidTag(String),
    MissingSeparator,
    MissingFileSystem,
    MissingMountSource,
    MissingSuperOpts,

    IoError(io::ErrorKind),
    ParseIntError(ParseIntError),
}

impl fmt::Display for LineError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        let desc: String = match self {
            LineError::MissingSpec => "Missing field #1 (spec)".into(),
            LineError::MissingFile => "Missing field #2 (file)".into(),
            LineError::InvalidFilePath(ref f) => {
                format!("Bad field #2 (file) value (not absolute path): {}", f).into()
            }
            LineError::InvalidFile(ref f) => format!("Bad field #2 (file) value: {}", f).into(),
            LineError::MissingVfstype => "Missing field #3 (vfstype)".into(),
            LineError::MissingMntops => "Missing field #4 (mntops)".into(),
            LineError::MissingFreq => "Missing field #5 (freq)".into(),
            LineError::InvalidFreq(ref f) => format!("Bad field #5 (dump) value: {}", f).into(),
            LineError::MissingPassno => "Missing field #6 (passno)".into(),
            LineError::InvalidPassno(ref f) => format!("Bad field #6 (passno) value: {}", f).into(),

            LineError::MissingMountId => "Missing field #1 (mount id)".into(),
            LineError::MissingParentId => "Missing field #2 (parent id)".into(),
            LineError::MissingDevId => "Missing field #3 (dev id)".into(),
            LineError::InvalidDevId(ref s) => format!("Invalid field #3 (dev id): {}", s),
            LineError::MissingRoot => "Missing field #4 (root)".into(),
            LineError::MissingMountPoint => "Missing field #5 (mount point)".into(),
            LineError::InvalidMountPoint(ref s) => format!("Invalid field #5 (mount point): {}", s),
            LineError::MissingMountOpts => "Missing field #6 (mount opts)".into(),
            LineError::InvalidTag(ref s) => format!("Invalid field #7 (mount opts): {}", s),
            LineError::MissingSeparator => "Missing field #8 (separator)".into(),
            LineError::MissingFileSystem => "Missing field #9 (filesystem)".into(),
            LineError::MissingMountSource => "Missing field #10 (mount source)".into(),
            LineError::MissingSuperOpts => "Missing field #11 (super opts)".into(),

            LineError::IoError(ref err) => format!("read line failed, {:?}", err),
            LineError::ParseIntError(ref err) => format!("Invalid integer, {}", err),
        };
        write!(out, "Line parsing: {}", desc)
    }
}
