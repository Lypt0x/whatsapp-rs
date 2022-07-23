use std::sync::atomic::AtomicU32;

pub struct Session {
    pub write_key: [u8; 32],
    pub read_key: [u8; 32],

    pub read_cnt: AtomicU32,
    pub write_cnt: AtomicU32
}