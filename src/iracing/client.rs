use crate::iracing::util::{SafeFileView, SafeHandle};
use crate::iracing::SimState;
use crate::iracing_basic_solution::header::{Header, VarBuf, VarHeader, VarHeaderRaw, VarHeaders};
use std::collections::HashMap;
use std::slice::from_raw_parts;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::task::spawn_blocking;
use windows::core::PCSTR;
use windows::Win32::System::Memory::{MapViewOfFile, OpenFileMappingA, FILE_MAP_READ};
use windows::Win32::System::Threading::{
    OpenEventA, WaitForSingleObject, SYNCHRONIZATION_SYNCHRONIZE,
};
use windows::Win32::System::WindowsProgramming::INFINITE;

static DATAVALIDEVENTNAME: &[u8] = b"Local\\IRSDKDataValidEvent";
static MEMMAPFILENAME: &[u8] = b"Local\\IRSDKMemMapFileName";

const STATUS_CONNECTED_FLAG: i32 = 1;

const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);

pub struct Client {
    vars_at_buf_len: i32,
    vars: Arc<VarHeaders>,
    last_tick_count: i32,
    last_valid_time: Option<SystemTime>,

    shared_memory: SharedMemory,
    data_valid_event: DataValidEvent,
}

impl Client {
    pub async fn connect() -> Self {
        let shared_memory = SharedMemory::connect();
        let data_valid_event = DataValidEvent::connect();

        let shared_memory = shared_memory.await;
        let data_valid_event = data_valid_event.await;

        while !shared_memory.is_header_connected() {
            data_valid_event.wait().await;
        }

        Client {
            vars_at_buf_len: -1,
            vars: Arc::new(HashMap::new()),
            last_tick_count: i32::MAX,
            last_valid_time: None,
            shared_memory,
            data_valid_event,
        }
    }

    pub async fn next_sim_state(&mut self) -> Option<SimState> {
        loop {
            if !self.is_connected() {
                return None;
            }

            if let Some(sim_state) = self.get_new_sim_state() {
                return Some(sim_state);
            }

            self.data_valid_event.wait().await;
        }
    }

    fn get_new_sim_state(&mut self) -> Option<SimState> {
        let header = self.shared_memory.header();

        if self.vars_at_buf_len != header.buf_len {
            self.vars = Arc::new(self.shared_memory.get_var_headers());
        }

        if header.status & STATUS_CONNECTED_FLAG == 0 {
            self.last_tick_count = i32::MAX;
            return None;
        }

        let mut latest_buffer_idx = 0;
        for idx in 1..(header.num_buf as usize) {
            if header.var_buf[latest_buffer_idx].tick_count < header.var_buf[idx].tick_count {
                latest_buffer_idx = idx;
            }
        }

        let buffer = &header.var_buf[latest_buffer_idx];

        if self.last_tick_count >= buffer.tick_count {
            self.last_tick_count = buffer.tick_count;
            return None;
        }

        // Two attempts to retrieve data
        for _ in 0..2 {
            let tick_count = buffer.tick_count;
            let data = self.shared_memory.data(header, buffer);
            if tick_count == buffer.tick_count {
                self.last_tick_count = tick_count;
                self.last_valid_time = Some(SystemTime::now());
                return if self.is_connected() {
                    Some(SimState::new(
                        header.clone(),
                        Arc::clone(&self.vars),
                        data.to_vec(),
                        self.shared_memory.raw_session_info().to_vec(),
                    ))
                } else {
                    None
                };
            }
        }

        None
    }

    fn is_connected(&self) -> bool {
        if !self.shared_memory.is_header_connected() {
            return false;
        }
        let last_valid_time = match self.last_valid_time {
            Some(v) => v,
            None => {
                return true;
            }
        };
        let elapsed = match SystemTime::now().duration_since(last_valid_time) {
            Ok(v) => v,
            Err(_) => return false,
        };
        elapsed < CLIENT_TIMEOUT
    }
}

struct SharedMemory {
    _handle: SafeHandle,
    file_view: SafeFileView,
}

impl SharedMemory {
    async fn connect() -> Self {
        let poll_delay = Duration::from_millis(250);

        let handle;
        loop {
            match unsafe {
                OpenFileMappingA(
                    FILE_MAP_READ.0,
                    false,
                    PCSTR::from_raw(MEMMAPFILENAME.as_ptr()),
                )
            }
            .ok()
            .and_then(SafeHandle::new)
            {
                Some(val) => {
                    handle = val;
                    break;
                }
                None => tokio::time::sleep(poll_delay).await,
            }
        }

        let file_view;
        loop {
            match SafeFileView::new(unsafe { MapViewOfFile(handle.get(), FILE_MAP_READ, 0, 0, 0) })
            {
                Some(val) => {
                    file_view = val;
                    break;
                }
                None => tokio::time::sleep(poll_delay).await,
            }
        }

        Self {
            _handle: handle,
            file_view,
        }
    }

    fn header(&self) -> &Header {
        unsafe { &*(self.file_view.get() as *const Header) }
    }

    fn is_header_connected(&self) -> bool {
        (self.header().status & STATUS_CONNECTED_FLAG) != 0
    }

    fn raw_var_headers(&self) -> &[VarHeaderRaw] {
        let header = self.header();
        unsafe {
            from_raw_parts(
                (self.file_view.get() as *const u8).offset(header.var_header_offset as isize)
                    as *const VarHeaderRaw,
                header.num_vars as usize,
            )
        }
    }

    fn get_var_headers(&self) -> VarHeaders {
        self.raw_var_headers()
            .iter()
            .filter_map(|var_header_raw| {
                let var_header = VarHeader::from_raw(var_header_raw)?;
                Some((var_header.name.clone(), var_header))
            })
            .collect()
    }

    fn data(&self, header: &Header, buffer: &VarBuf) -> &[u8] {
        unsafe {
            from_raw_parts(
                (self.file_view.get() as *const u8).offset(buffer.buf_offset as isize),
                header.buf_len as usize,
            )
        }
    }

    fn raw_session_info(&self) -> &[u8] {
        let header = self.header();
        unsafe {
            from_raw_parts(
                (self.file_view.get() as *const u8).offset(header.session_info_offset as isize),
                header.session_info_len as usize,
            )
        }
    }
}

struct DataValidEvent {
    handle: SafeHandle,
}

impl DataValidEvent {
    async fn connect() -> Self {
        let poll_delay = Duration::from_millis(250);
        loop {
            match unsafe {
                OpenEventA(
                    SYNCHRONIZATION_SYNCHRONIZE,
                    false,
                    PCSTR::from_raw(DATAVALIDEVENTNAME.as_ptr()),
                )
            }
            .ok()
            .and_then(SafeHandle::new)
            {
                Some(handle) => {
                    return Self { handle };
                }
                None => tokio::time::sleep(poll_delay).await,
            }
        }
    }

    #[allow(dead_code)]
    fn wait_with_timeout(&self, timeout: Duration) {
        unsafe {
            WaitForSingleObject(self.handle.get(), timeout.as_millis() as u32);
        }
    }

    async fn wait(&self) {
        let handle = self.handle.get();
        // TODO: show some warning on error
        spawn_blocking(move || unsafe {
            WaitForSingleObject(handle, INFINITE);
        })
        .await
        .ok();
    }
}