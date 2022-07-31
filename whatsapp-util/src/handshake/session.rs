pub struct Session {
    pub write_key: [u8; 32],
    pub read_key: [u8; 32],

    pub read_cnt: u64,
    pub write_cnt: u64,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            write_key: [0u8; 32],
            read_key: [0u8; 32],
            read_cnt: 0,
            write_cnt: 0,
        }
    }
}
