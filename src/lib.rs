#![no_std]

pub use cfmt_macros::{
    print, println, eprint, eprintln,
    cprint, cprintln, ceprint, ceprintln
};

#[inline(never)]
pub fn encode_utf8(c: char, buf: &mut [u8; 5]) -> *const u8 {
    let mut u = c as u32;
    buf[4] = 0_u8;
    if u <= 0x7F {
        buf[3] = u as u8;
        return &buf[3] as *const u8;
    }
    buf[3] = (u as u8 & 0x3F) | 0x80;
    u >>= 6;
    if u <= 0x1F {
        buf[2] = (u | 0xC0) as u8;
        return &buf[2] as *const u8;
    }
    buf[2] = (u as u8 & 0x3F) | 0x80;
    u >>= 6;
    if u <= 0xF {
        buf[1] = (u | 0xE0) as u8;
        return &buf[1] as *const u8;
    } 
    buf[1] = (u as u8 & 0x3F) | 0x80;
    u >>= 6;
    buf[0] = (u | 0xF0) as u8;
    return buf as *const u8;
}

