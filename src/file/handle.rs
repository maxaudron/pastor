use std::{
    ops::{Deref, DerefMut},
    os::fd::AsRawFd,
};

use tokio::fs::File;
use xattr::FileExt;

use crate::file::PasteError;

pub struct PasteHandle {
    file: File,
}

impl PasteHandle {
    pub fn set_xattr_i64(&self, name: &'static str, value: i64) -> Result<(), PasteError> {
        self.set_xattr(name, &value.to_be_bytes())?;
        Ok(())
    }

    pub fn set_xattr_str(&self, name: &'static str, value: &str) -> Result<(), PasteError> {
        self.set_xattr(name, value.as_bytes())?;
        Ok(())
    }

    pub fn get_xattr_i64(&self, name: &'static str) -> Result<i64, PasteError> {
        Ok(i64::from_be_bytes(
            self.get_xattr(name)?
                .ok_or(PasteError::XattrNotFound(name))?
                .try_into()
                .map_err(PasteError::ParseError)?,
        ))
    }

    pub fn get_xattr_str(&self, name: &'static str) -> Result<String, PasteError> {
        Ok(str::from_utf8(&self.get_xattr(name)?.ok_or(PasteError::XattrNotFound(name))?)?.to_owned())
    }

    pub fn into_file(self) -> File {
        self.file
    }
}

impl DerefMut for PasteHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.file
    }
}

impl Deref for PasteHandle {
    type Target = File;

    fn deref(&self) -> &Self::Target {
        &self.file
    }
}

impl From<File> for PasteHandle {
    fn from(file: File) -> Self {
        Self { file }
    }
}

impl AsRawFd for PasteHandle {
    fn as_raw_fd(&self) -> std::os::unix::prelude::RawFd {
        self.file.as_raw_fd()
    }
}

impl FileExt for PasteHandle {}
