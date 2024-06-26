pub const KEY_LEN: usize = 32;

// a list for each bit of the node ID
// 32*8 --> 256
pub const N_BUCKETS: usize = KEY_LEN * 8;

// number entries in a list
pub const K_PARAM: usize = 20;

// buffer size used for streaming UDP
pub const BUF_SIZE: usize = 4096 * 2;

// response timeout 5000ms
pub const TIMEOUT: u64 = 5000;

// number of concurrent lookups in node lookup
pub const ALPHA: usize = 3;

pub const VERBOSE: bool = false;