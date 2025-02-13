//! hifmt - Format output without Rust code segment in binary
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
//!     #[cfg(not(feature = "nolibc"))]
//!     fn snprintf(buf: *mut u8, size: usize, format: *const u8, ...) -> i32;
//! }
//!
//! #[cfg(feature = "nolibc")]
//! fn write_buf(buf: &[u8]) -> usize {
//!     unsafe { dprintf(1, b"%.*s\0".as_ptr(), buf.len() as i32, buf.as_ptr()) as usize }
//! }
//! #[cfg(feature = "nolibc")]
//! hifmt::make_nolibc_formatter!(write_buf);
//!
//! hifmt::println!("hello world");
//! hifmt::println!("signed decimal {:d}", -1);
//! hifmt::println!("unsigned decimal {:u}", -1);
//! hifmt::println!("hexadecimal {:x}", -1);
//! hifmt::println!("pointer {:p}", &1);
//! hifmt::println!("float {:e}", -1.0);
//! hifmt::println!("rust &str {:rs}", "hello world");
//! hifmt::println!("rust &[u8] {:rb}", b"hello world");
//! hifmt::println!("rust char {:rc}", '中');
//! hifmt::println!("c str {:cs}", b"hello world\0");
//! hifmt::println!("c char {:cc}", b'0');
//!
//! let mut buf = [0_u8; 100];
//! hifmt::bprint!(&mut buf, "snprintf rust string {:rs}", "hello world");
//! hifmt::println!("c str {:cs}", &buf);
//!
//! //test_retn_length
//! let s = "hello world";
//! let len = hifmt::bprint!(&mut [], "{:rs}", s);
//! assert_eq!(len as usize, s.len());
//! let len = hifmt::print!("{:rs}", s);
//! assert_eq!(len as usize, s.len());
//!
//!
//! // test_fat_pointer
//! let mut buf = [0_u8; 100];
//! let s = "hello";
//! let n = s as *const _ as *const u8 as usize;
//! hifmt::bprint!(&mut buf[0..], "{:p} {:p}", s, s);
//! let s = format!("0x{:x} 0x{:x}\0", n, n);
//! assert_eq!(s.as_bytes(), &buf[0..s.len()]);
//! ```
//!

//#![no_std]

#[cfg(not(feature = "nolibc"))]
mod libc;
#[cfg(not(feature = "nolibc"))]
pub use libc::*;

#[cfg(feature = "nolibc")]
mod nolibc;
#[cfg(feature = "nolibc")]
pub use nolibc::*;

#[inline(never)]
pub fn encode_utf8(c: char, buf: &mut [u8; 4]) -> &[u8] {
    let mut u = c as u32;
    if u <= 0x7F {
        buf[3] = u as u8;
        return &buf[3..];
    }
    buf[3] = (u as u8 & 0x3F) | 0x80;
    u >>= 6;
    if u <= 0x1F {
        buf[2] = (u | 0xC0) as u8;
        return &buf[2..];
    }
    buf[2] = (u as u8 & 0x3F) | 0x80;
    u >>= 6;
    if u <= 0xF {
        buf[1] = (u | 0xE0) as u8;
        return &buf[1..];
    }
    buf[1] = (u as u8 & 0x3F) | 0x80;
    u >>= 6;
    buf[0] = (u | 0xF0) as u8;
    buf
}

