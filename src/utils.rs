pub(crate) fn from_le_bytes(x: &[u8]) -> u32 {
    u32::from_le_bytes([x[0], x[1], x[2], x[3]])
}

pub(crate) fn const_time_eq(a: &[u8], b: &[u8]) -> bool {
    let mut temp = 0;

    for (i, j) in a.iter().zip(b.iter()) {
        temp |= i ^ j;
    }

    temp == 0
}
