#[derive(Debug, Clone)]
pub struct ClockData {
    pub html: bytes::Bytes,
    pub svg: bytes::Bytes,
    pub select: bytes::Bytes,
    pub gif: bytes::Bytes,
    pub rtl: bytes::Bytes,
}

#[derive(Debug, Clone)]
pub struct Context {
    pub previous_timestamp: i64,
    pub timestamp: i64,
    pub connection_count: usize,
    pub jst: String,
}
