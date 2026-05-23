use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

pub fn push_ext<S>(path: &PathBuf, ext: S) -> PathBuf
where
    S: AsRef<OsStr>,
{
    let mut s = OsString::from(path);
    s.push(ext);
    PathBuf::from(s)
}
