use std::sync::atomic::AtomicU32;

pub struct Session {
    pub write_key: [u8; 32],
    pub read_key: [u8; 32],

    pub read_cnt: AtomicU32,
    pub write_cnt: AtomicU32,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            write_key: [0u8; 32],
            read_key: [0u8; 32],
            read_cnt: AtomicU32::new(0),
            write_cnt: AtomicU32::new(0),
        }
    }
}
