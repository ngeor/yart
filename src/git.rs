//! Calls git as a process

use std::ffi::OsStr;
use std::fmt::Formatter;
use std::path::Path;
use std::process::Command;
use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum GitError {
    IOError(std::io::Error),
    FromUtf8Error(FromUtf8Error),
    NonZeroExitCode,
}

impl std::fmt::Display for GitError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IOError(e) => std::fmt::Display::fmt(&e, f),
            Self::FromUtf8Error(e) => std::fmt::Display::fmt(&e, f),
            Self::NonZeroExitCode => f.write_str("git returned non-zero exit code"),
        }
    }
}

impl std::error::Error for GitError {}

pub fn tags<P: AsRef<Path>>(dir: P) -> Result<String, GitError> {
    match Command::new("git")
        .arg("tag")
        .arg("--list")
        .current_dir(dir)
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                match String::from_utf8(output.stdout) {
                    Ok(s) => Ok(s),
                    Err(err) => Err(GitError::FromUtf8Error(err)),
                }
            } else {
                Err(GitError::NonZeroExitCode)
            }
        }
        Err(err) => Err(GitError::IOError(err)),
    }
}

pub fn add<P: AsRef<Path>, Q: AsRef<OsStr>>(dir: P, item_to_add: Q) -> Result<(), GitError> {
    discard_output(
        Command::new("git")
            .arg("add")
            .arg(item_to_add)
            .current_dir(dir),
    )
}

pub fn commit<P: AsRef<Path>, Q: AsRef<OsStr>>(dir: P, message: Q) -> Result<(), GitError> {
    discard_output(
        Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(message)
            .current_dir(dir),
    )
}

pub fn tag<P: AsRef<Path>, Q: AsRef<OsStr>, R: AsRef<OsStr>>(
    dir: P,
    message: Q,
    tag: R,
) -> Result<(), GitError> {
    discard_output(
        Command::new("git")
            .arg("tag")
            .arg("-m")
            .arg(message)
            .arg(tag)
            .current_dir(dir),
    )
}

pub fn push<P: AsRef<Path>>(dir: P) -> Result<(), GitError> {
    discard_output(
        Command::new("git")
            .arg("push")
            .arg("--follow-tags")
            .current_dir(dir),
    )
}

fn discard_output(command: &mut Command) -> Result<(), GitError> {
    match command.output() {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                Err(GitError::NonZeroExitCode)
            }
        }
        Err(err) => Err(GitError::IOError(err)),
    }
}
