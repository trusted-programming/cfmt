//! cfmt - Format output without Rust code segment in binary
//! 
//! # Design objective:
//! 1. The print output depends on the API of C
//! 2. Eliminate the dependency on Display/Debug trait.
//!
//! # Examples
//!
//! ```rust
//! #[link(name = "c")]
//! extern "C" {
//!     fn dprintf(fd: i32, format: *const u8, ...) -> i32;
//!     fn snprintf(buf: *mut u8, size: usize, format: *const u8, ...) -> i32;
//! }
//! orion_cfmt::println!("hello world");
//! orion_cfmt::println!("signed decimal {:d}", -1);
//! orion_cfmt::println!("unsigned decimal {:u}", -1);
//! orion_cfmt::println!("hexadecimal {:x}", -1);
//! orion_cfmt::println!("pointer {:p}", &1);
//! orion_cfmt::println!("float {:e}", -1.0);
//! orion_cfmt::println!("rust &str {:rs}", "hello world");
//! orion_cfmt::println!("rust &[u8] {:rb}", b"hello world");
//! orion_cfmt::println!("rust char {:rc}", '中');
//! orion_cfmt::println!("c str {:cs}", b"hello world\0");
//! orion_cfmt::println!("c char {:cc}", b'0');
//!
//! let mut buf = [0_u8; 100];
//! orion_cfmt::bprint!(&mut buf, "snprintf rust string {:rs}", "hello world");
//! orion_cfmt::println!("c str {:cs}", &buf);
//!

#![no_std]

pub use cfmt_macros::{
    print, println, eprint, eprintln,
    cprint, cprintln, ceprint, ceprintln,
    csprint, cbprint, sprint, bprint
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

#[cfg(test)]
mod test {
    extern crate std;
    use std::format;
    #[link(name = "c")]
    extern "C" {
        fn snprintf(buf: *mut u8, len: usize, format: *const u8, ...) -> i32;
    }

    #[test]
    fn test_fat_pointer() {
        let mut buf = [0_u8; 100];
        let s = "hello";
        let n = s as *const _ as *const u8 as usize;
        super::bprint!(&mut buf[0..], "{:p} {:p}", s, s);
        let s = format!("0x{:x} 0x{:x}\0", n, n);
        assert_eq!(s.as_bytes(), &buf[0..s.len()]);
    }
}
