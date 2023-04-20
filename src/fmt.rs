use crate::const_string::ConstString;

pub(crate) const fn fmt_u64(buf: &mut ConstString, n: u64) {
    const ASCII_NUM_BASE: u8 = 48u8;

    let n_digits = n.ilog10() + 1;

    buf.reserve(n_digits as usize);

    let mut curr = n;

    while curr > 0 {
        buf.push_ascii(ASCII_NUM_BASE + (curr % 10).to_le_bytes()[0]);

        curr /= 10;
    }
}
