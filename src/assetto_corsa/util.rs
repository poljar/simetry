use crate::assetto_corsa::shared_memory_data::StatusRaw;
use std::fmt::Debug;
use std::sync::Arc;

#[repr(C, packed(4))]
#[derive(Clone, Debug)]
pub struct PageFileGraphicsTop {
    pub packet_id: i32,
    pub status: StatusRaw,
}

#[repr(C, packed(4))]
#[derive(Clone, Debug)]
pub struct PageFileStaticTop {
    pub sm_version: [u16; 15],
}

pub trait WithPacketId {
    fn packet_id(&self) -> i32;
}

pub trait AcApiVersion: Debug {
    const MAJOR_MIN: u16;
    const MAJOR_MAX: u16;
    const MINOR_MIN: u16;
    const MINOR_MAX: u16;
    type PageStatic: Clone + Into<Self::DataStatic>;
    type DataStatic: Clone + Debug;
    type PagePhysics: Clone + WithPacketId + Into<Self::DataPhysics>;
    type DataPhysics: Clone + WithPacketId + Debug;
    type PageGraphics: Clone + WithPacketId + Into<Self::DataGraphics>;
    type DataGraphics: Clone + WithPacketId + Debug;
}

#[derive(Clone, Debug)]
pub struct SimState<Version: AcApiVersion> {
    pub static_data: Arc<Version::DataStatic>,
    pub physics: Arc<Version::DataPhysics>,
    pub graphics: Arc<Version::DataGraphics>,
}

fn check_version<Version: AcApiVersion>(data: &PageFileStaticTop) -> anyhow::Result<()> {
    let sm_version_string = super::conversions::extract_string(&data.sm_version);
    let sm_version = sm_version_string
        .split('.')
        .map(|v| v.parse::<u16>())
        .collect::<Result<Vec<u16>, _>>()
        .with_context(|| format!("Invalid shared memory version string: {sm_version_string:?}"))?;
    let Some(major) = sm_version.first().copied() else {
        bail!("Shared memory version is missing major version in: {sm_version_string:?}");
    };
    let Some(minor) = sm_version.get(1).copied() else {
        bail!("Shared memory version is missing minor version in: {sm_version_string:?}");
    };
    let min_version = ((Version::MAJOR_MIN as u32) << 16) | (Version::MINOR_MIN as u32);
    let max_version = ((Version::MAJOR_MAX as u32) << 16) | (Version::MINOR_MAX as u32);
    let version = ((major as u32) << 16) | (minor as u32);
    if version < min_version || version > max_version {
        bail!(
            "Expected shared memory major version in {}.{} - {}.{} range, got {}.{}",
            Version::MAJOR_MIN,
            Version::MINOR_MIN,
            Version::MAJOR_MAX,
            Version::MINOR_MAX,
            major,
            minor,
        );
    }
    Ok(())
}

use anyhow::{bail, Context};
#[cfg(target_family = "unix")]
pub use unix::SharedMemoryClient;
#[cfg(target_family = "windows")]
pub use windows::SharedMemoryClient;

#[cfg(target_family = "unix")]
mod unix {
    use anyhow::{Context, Result};
    use libc::{c_void, shm_open};
    use std::{
        ffi::CString,
        marker::PhantomData,
        os::fd::{AsRawFd, FromRawFd, OwnedFd},
        sync::Arc,
        time::Duration,
    };

    use crate::assetto_corsa::{
        util::{check_version, WithPacketId},
        Status,
    };

    use super::{AcApiVersion, PageFileGraphicsTop, PageFileStaticTop, SimState};

    struct SharedMemory<T> {
        _fd: OwnedFd,
        memory: *mut c_void,
        phantom_data: PhantomData<T>,
    }

    // Send + Sync here is fine because we own the file descriptor and pointer to the mmapped region
    // and we're only reading from it.
    unsafe impl<T: Send + std::fmt::Debug> Send for SharedMemory<T> {}
    unsafe impl<T: Sync + std::fmt::Debug> Sync for SharedMemory<T> {}

    impl<T> SharedMemory<T> {
        pub fn connect(foo: &str) -> Result<Self> {
            let path = CString::new(foo).expect("We should be able to build this static C string");

            let fd = unsafe { shm_open(path.as_ptr(), libc::SHM_RDONLY, 0) };

            if fd == -1 {
                Err(std::io::Error::last_os_error())
                    .context(format!("Opening the {path:?} file failed"))
            } else {
                let len = std::mem::size_of::<T>();
                let fd = unsafe { OwnedFd::from_raw_fd(fd) };

                let memory = unsafe {
                    libc::mmap(
                        std::ptr::null_mut(),
                        len,
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

        pub fn get(&self) -> &T {
            unsafe { &(*(self.memory as *const T)) }
        }
    }

    pub struct SharedMemoryClient<Version: AcApiVersion> {
        static_data: Version::DataStatic,
        graphics_data: Version::DataGraphics,
        physics_data: Version::DataPhysics,
        static_data_memory: SharedMemory<Version::PageStatic>,
        graphics_data_memory: SharedMemory<Version::PageGraphics>,
        physics_data_memory: SharedMemory<Version::PagePhysics>,
        graphics_data_memory_top: SharedMemory<PageFileGraphicsTop>,
    }

    impl<Version: AcApiVersion> SharedMemoryClient<Version> {
        pub fn get_version(&self) -> u16 {
            Version::MAJOR_MIN
        }

        pub async fn connect(retry_delay: Duration) -> Self {
            loop {
                if let Ok(v) = Self::try_connect().await {
                    return v;
                }
                tokio::time::sleep(retry_delay).await;
            }
        }

        pub async fn next_sim_state(&mut self) -> Option<SimState<Version>> {
            loop {
                if !Self::is_connected(&self.graphics_data_memory_top) {
                    return None;
                }

                let mut changed = false;

                let static_data = self.static_data_memory.get();
                self.static_data = static_data.clone().into();

                let physics_data = self.physics_data_memory.get();

                if self.physics_data.packet_id() != physics_data.packet_id() {
                    changed = true;
                    self.physics_data = physics_data.clone().into();
                }

                let graphics_data = self.graphics_data_memory.get();

                if self.graphics_data.packet_id() != physics_data.packet_id() {
                    changed = true;
                    self.graphics_data = graphics_data.clone().into();
                }

                if changed {
                    return Some(SimState {
                        static_data: Arc::new(self.static_data.clone()),
                        physics: Arc::new(self.physics_data.clone()),
                        graphics: Arc::new(self.graphics_data.clone()),
                    });
                } else {
                    tokio::time::sleep(Duration::from_millis(2)).await;
                }
            }
        }

        fn check_version(page_file_top: &SharedMemory<PageFileStaticTop>) -> Result<()> {
            let data = page_file_top.get();
            check_version::<Version>(data)
        }

        fn is_connected(graphics_data: &SharedMemory<PageFileGraphicsTop>) -> bool {
            let status: Status = graphics_data.get().status.into();
            status != Status::Off
        }

        pub async fn try_connect() -> Result<Self> {
            let graphics_data_memory_top: SharedMemory<PageFileGraphicsTop> =
                SharedMemory::connect("/acpmf_graphics")?;

            while !Self::is_connected(&graphics_data_memory_top) {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }

            let page_file_top: SharedMemory<PageFileStaticTop> =
                SharedMemory::connect("/acpmf_static")?;
            Self::check_version(&page_file_top)?;

            let static_data_memory: SharedMemory<Version::PageStatic> =
                SharedMemory::connect("/acpmf_static")?;
            let graphics_data_memory: SharedMemory<Version::PageGraphics> =
                SharedMemory::connect("/acpmf_graphics")?;
            let physics_data_memory: SharedMemory<Version::PagePhysics> =
                SharedMemory::connect("/acpmf_physics")?;

            let static_data: Version::DataStatic = static_data_memory.get().clone().into();
            let graphics_data: Version::DataGraphics = graphics_data_memory.get().clone().into();
            let physics_data: Version::DataPhysics = physics_data_memory.get().clone().into();

            Ok(Self {
                static_data,
                graphics_data,
                physics_data,
                graphics_data_memory_top,
                static_data_memory,
                graphics_data_memory,
                physics_data_memory,
            })
        }

        pub fn static_data(&self) -> &Version::DataStatic {
            &self.static_data
        }
    }
}

#[cfg(target_family = "windows")]
mod windows {
    use super::{
        check_version, AcApiVersion, PageFileGraphicsTop, PageFileStaticTop, SimState, WithPacketId,
    };
    use crate::assetto_corsa::conversions::extract_string;
    use crate::assetto_corsa::Status;
    use crate::windows_util::SharedMemory;
    use anyhow::{bail, Context, Result};
    use std::sync::Arc;
    use std::time::Duration;

    pub struct SharedMemoryClient<Version: AcApiVersion> {
        static_data: Arc<Version::DataStatic>,
        physics_data: SharedMemory,
        graphics_data: SharedMemory,
        last_physics: Arc<Version::DataPhysics>,
        last_graphics: Arc<Version::DataGraphics>,
    }

    impl<Version: AcApiVersion> SharedMemoryClient<Version> {
        pub async fn connect(retry_delay: Duration) -> Self {
            loop {
                if let Ok(v) = Self::try_connect().await {
                    return v;
                }
                tokio::time::sleep(retry_delay).await;
            }
        }

        pub async fn try_connect() -> Result<Self> {
            let poll_delay = Duration::from_millis(250);
            let graphics_data = SharedMemory::connect(b"Local\\acpmf_graphics\0", poll_delay).await;
            while !Self::is_connected(&graphics_data) {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            let physics_data = SharedMemory::connect(b"Local\\acpmf_physics\0", poll_delay).await;
            let static_data = SharedMemory::connect(b"Local\\acpmf_static\0", poll_delay).await;

            Self::check_version(&static_data)?;
            let static_data = Arc::new(
                unsafe { static_data.get_as::<Version::PageStatic>() }
                    .clone()
                    .into(),
            );
            let last_physics = Arc::new(Self::physics(&physics_data));
            let last_graphics = Arc::new(Self::graphics(&graphics_data));
            Ok(Self {
                static_data,
                physics_data,
                graphics_data,
                last_physics,
                last_graphics,
            })
        }

        fn is_connected(graphics_data: &SharedMemory) -> bool {
            let status: Status = unsafe { graphics_data.get_as::<PageFileGraphicsTop>() }
                .status
                .into();
            status != Status::Off
        }

        fn check_version(static_memory: &SharedMemory) -> Result<()> {
            let sm_version = unsafe { static_memory.get_as::<PageFileStaticTop>() }.clone();
            check_version::<Version>(&sm_version)
        }

        pub async fn next_sim_state(&mut self) -> Option<SimState<Version>> {
            loop {
                if !Self::is_connected(&self.graphics_data) {
                    return None;
                }
                let mut changed = false;
                let physics_packet_id = unsafe {
                    self.physics_data
                        .get_as::<Version::PagePhysics>()
                        .packet_id()
                };
                if self.last_physics.packet_id() != physics_packet_id {
                    changed = true;
                    self.last_physics = Arc::new(Self::physics(&self.physics_data));
                }
                let graphics_packet_id = unsafe {
                    self.graphics_data
                        .get_as::<Version::PageGraphics>()
                        .packet_id()
                };
                if self.last_graphics.packet_id() != graphics_packet_id {
                    changed = true;
                    self.last_graphics = Arc::new(Self::graphics(&self.graphics_data));
                }
                if changed {
                    return Some(SimState {
                        static_data: Arc::clone(&self.static_data),
                        physics: Arc::clone(&self.last_physics),
                        graphics: Arc::clone(&self.last_graphics),
                    });
                } else {
                    tokio::time::sleep(Duration::from_millis(2)).await;
                }
            }
        }

        pub fn static_data(&self) -> &Version::DataStatic {
            &self.static_data
        }

        fn physics(physics_data: &SharedMemory) -> Version::DataPhysics {
            loop {
                let packet_id_1 =
                    unsafe { physics_data.get_as::<Version::PagePhysics>().packet_id() };
                let data = unsafe { physics_data.get_as::<Version::PagePhysics>() }.clone();
                let packet_id_2 =
                    unsafe { physics_data.get_as::<Version::PagePhysics>().packet_id() };
                if packet_id_1 == packet_id_2 {
                    return data.into();
                }
            }
        }

        fn graphics(graphics_data: &SharedMemory) -> Version::DataGraphics {
            loop {
                let packet_id_1 =
                    unsafe { graphics_data.get_as::<Version::PageGraphics>().packet_id() };
                let data = unsafe { graphics_data.get_as::<Version::PageGraphics>() }.clone();
                let packet_id_2 =
                    unsafe { graphics_data.get_as::<Version::PageGraphics>().packet_id() };
                if packet_id_1 == packet_id_2 {
                    return data.into();
                }
            }
        }
    }
}
