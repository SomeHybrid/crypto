pub(crate) fn from_le_bytes(x: &[u8]) -> u32 {
    u32::from_le_bytes([x[0], x[1], x[2], x[3]])
}
