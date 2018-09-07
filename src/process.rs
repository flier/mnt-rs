use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::PathBuf;
use std::str::FromStr;

pub type MountId = usize;

use {LineError, MntOps};

const PROC_SELF_MOUNTINFO: &'static str = "/proc/self/mountinfo";

/// Returns the mount points in the current process mount namespace
pub fn self_mountinfo() -> io::Result<impl Iterator<Item = Result<MountEntry, LineError>>> {
    File::open(PROC_SELF_MOUNTINFO).map(parse_mountinfo)
}

/// Returns the mount points in the given process mount namespace
pub fn proc_mountinfo(pid: u32) -> io::Result<impl Iterator<Item = Result<MountEntry, LineError>>> {
    File::open(format!("/proc/{}/mountinfo", pid)).map(parse_mountinfo)
}

/// Parse the mount points from buffer
pub fn parse_mountinfo<R: Read>(r: R) -> impl Iterator<Item = Result<MountEntry, LineError>> {
    let r = BufReader::new(r);

    r.lines()
        .map(|line| line.map_err(|err| LineError::IoError(err.kind()))?.parse())
}

/// The mount points in the process's mount namespace
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MountEntry {
    /// a unique ID for the mount
    pub mount_id: MountId,
    /// the ID of the parent mount
    pub parent_id: MountId,
    /// the major version of st_dev for files on this filesystem
    pub dev_major: u32,
    /// the minor version of st_dev for files on this filesystem
    pub dev_minor: u32,
    /// the pathname of the directory in the filesystem which forms the root of this mount.
    pub root: PathBuf,
    /// the pathname of the mount point relative to the process's root directory.
    pub mount_point: PathBuf,
    /// per-mount options.
    pub mount_opts: Vec<MntOps>,
    /// zero or more fields.
    pub tags: Vec<(String, Option<String>)>,
    /// the filesystem type.
    pub filesystem: String,
    /// filesystem-specific information or "none".
    pub mount_source: String,
    /// per-superblock options.
    pub super_opts: Vec<MntOps>,
}

/// 36 35 98:0 /mnt1 /mnt2 rw,noatime master:1 - ext3 /dev/root rw,errors=continue
/// (1)(2)(3)   (4)   (5)      (6)      (7)   (8) (9)   (10)         (11)
impl FromStr for MountEntry {
    type Err = LineError;

    fn from_str(line: &str) -> Result<MountEntry, LineError> {
        let line = line.trim();
        let mut tokens = line
            .split_terminator(|s: char| s == ' ' || s == '\t')
            .filter(|s| s != &"");

        let mount_id = tokens
            .next()
            .ok_or(LineError::MissingMountId)
            .and_then(|s| s.parse().map_err(LineError::ParseIntError))?;
        let parent_id = tokens
            .next()
            .ok_or(LineError::MissingParentId)
            .and_then(|s| s.parse().map_err(LineError::ParseIntError))?;
        let (dev_major, dev_minor) = tokens.next().ok_or(LineError::MissingDevId).and_then(
            |dev| {
                let mut tokens = dev.split_terminator(':');

                let dev_major = tokens
                    .next()
                    .ok_or_else(|| LineError::InvalidDevId(dev.to_owned()))?
                    .parse()
                    .map_err(LineError::ParseIntError)?;
                let dev_minor = tokens
                    .next()
                    .ok_or_else(|| LineError::InvalidDevId(dev.to_owned()))?
                    .parse()
                    .map_err(LineError::ParseIntError)?;

                if tokens.next().is_none() {
                    Ok((dev_major, dev_minor))
                } else {
                    Err(LineError::InvalidDevId(dev.to_owned()))
                }
            },
        )?;
        let root = tokens.next().ok_or(LineError::MissingRoot)?.into();
        let mount_point = tokens.next().ok_or(LineError::MissingMountPoint)?.into();
        let mount_opts = tokens
            .next()
            .ok_or(LineError::MissingMountOpts)?
            .split_terminator(',')
            .map(|s| s.parse())
            .collect::<Result<Vec<_>, _>>()?;

        let mut tags = vec![];

        loop {
            match tokens.next() {
                Some("-") => break,
                Some(tag) => {
                    let mut tokens = tag.split_terminator(':');

                    let name = tokens.next().ok_or(LineError::InvalidTag(tag.to_owned()))?;
                    let value = tokens.next().map(|s| s.to_owned());

                    tags.push((name.to_owned(), value));
                }
                None => {
                    return Err(LineError::MissingSeparator);
                }
            }
        }

        let filesystem = tokens
            .next()
            .map(|s| s.to_owned())
            .ok_or(LineError::MissingFileSystem)?;
        let mount_source = tokens
            .next()
            .map(|s| s.to_owned())
            .ok_or(LineError::MissingMountSource)?;
        let super_opts = tokens
            .next()
            .ok_or(LineError::MissingSuperOpts)?
            .split_terminator(',')
            .map(|s| s.parse())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(MountEntry {
            mount_id,
            parent_id,
            dev_major,
            dev_minor,
            root,
            mount_point,
            mount_opts,
            tags,
            filesystem,
            mount_source,
            super_opts,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "linux")]
    #[test]
    fn test_self_mountinfo() {
        assert!(self_mountinfo().is_ok());
    }

    #[test]
    fn test_parse_mountinfo() {
        let s = b"21 26 0:20 / /sys rw,nosuid,nodev,noexec,relatime shared:7 - sysfs sysfs rw
26 0 8:2 / / rw,relatime - ext4 /dev/sda2 rw,data=ordered";

        let mut entries = parse_mountinfo(&s[..]);

        assert_eq!(
            MountEntry {
                mount_id: 21,
                parent_id: 26,
                dev_major: 0,
                dev_minor: 20,
                root: "/".into(),
                mount_point: "/sys".into(),
                mount_opts: vec![
                    MntOps::Write(true),
                    MntOps::Suid(false),
                    MntOps::Dev(false),
                    MntOps::Exec(false),
                    MntOps::RelAtime(true),
                ],
                tags: vec![("shared".to_owned(), Some("7".to_owned()))],
                filesystem: "sysfs".to_owned(),
                mount_source: "sysfs".to_owned(),
                super_opts: vec![MntOps::Write(true)],
            },
            entries.next().unwrap().unwrap()
        );
        assert_eq!(
            MountEntry {
                mount_id: 26,
                parent_id: 0,
                dev_major: 8,
                dev_minor: 2,
                root: "/".into(),
                mount_point: "/".into(),
                mount_opts: vec![MntOps::Write(true), MntOps::RelAtime(true)],
                tags: vec![],
                filesystem: "ext4".to_owned(),
                mount_source: "/dev/sda2".to_owned(),
                super_opts: vec![
                    MntOps::Write(true),
                    MntOps::Extra("data=ordered".to_owned()),
                ],
            },
            entries.next().unwrap().unwrap()
        );

        assert!(entries.next().is_none());
    }
}
