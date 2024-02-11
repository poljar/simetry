#![allow(dead_code)]

use std::{
    ffi::{c_void, CString},
    marker::PhantomData,
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
};

use anyhow::{Context, Result};
use libc::shm_open;

pub struct SharedMemory<T> {
    _fd: OwnedFd,
    memory: *mut c_void,
    phantom_data: PhantomData<T>,
}

// Send + Sync here is fine because we own the file descriptor and pointer to the mmapped region
// and we're only reading from it.
unsafe impl<T: Send + std::fmt::Debug> Send for SharedMemory<T> {}
unsafe impl<T: Sync + std::fmt::Debug> Sync for SharedMemory<T> {}

impl<T> SharedMemory<T> {
    pub fn open(name: &str) -> Result<Self> {
        // This only fails if the bytes passed to the `new()` method contain a 0 byte, since str is
        // guaranteed to have UTF-8, this can't be the case.
        let path = CString::new(name).expect("We should be able to convert a str into a CString");

        let fd = unsafe { shm_open(path.as_ptr(), libc::SHM_RDONLY, 0) };

        if fd == -1 {
            Err(std::io::Error::last_os_error())
                .context(format!("Opening the {path:?} file failed"))
        } else {
            let fd = unsafe { OwnedFd::from_raw_fd(fd) };
            let length = std::mem::size_of::<T>();

            let memory = unsafe {
                libc::mmap(
                    std::ptr::null_mut(),
                    length,
                    libc::PROT_READ,
                    libc::MAP_SHARED,
                    fd.as_raw_fd(),
                    0,
                )
            };

            if memory == libc::MAP_FAILED {
                Err(std::io::Error::last_os_error())
                    .context("Unable to mmap the opened SHM file {path}")
            } else {
                Ok(Self {
                    _fd: fd,
                    memory,
                    phantom_data: Default::default(),
                })
            }
        }
    }

    pub unsafe fn get(&self) -> &T {
        &(*(self.memory as *const T))
    }
}
