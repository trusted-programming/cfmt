/// feature = "nolibc"
/// 用户必须提供一个字符串输出函数: fn(&[u8]) -> usize
/// 这里将此输出函数适配到`hifmt::Formatter`用户`hifmt::print`系列, 适用于无多线程并发输出场景.
#[macro_export]
macro_rules! make_nolibc_formatter {
    ($printf: ident) => {
        #[allow(non_camel_case_types)]
        struct _hifmt_Formatter;
        impl $crate::Formatter for _hifmt_Formatter {
            fn new(_: i32) -> Self {
                _hifmt_Formatter
            }
            fn write_buf(&mut self, buf: &[u8]) -> usize {
                $printf(buf)
            }
        }
    };
}

/// feature = "nolibc"
/// 用户实现支持hifmt::Formater的类型用于`hifmt::print`系列.
/// 如果打印输出有多线程同步需求，应该完整实现`Formatter`接口并接口`nolibc_formatter`使用.
#[macro_export]
macro_rules! nolibc_formatter {
    ($formatter: path) => {
        #[allow(non_camel_case_types)]
        type _hifmt_Formatter = $formatter;
    };
}

pub trait Formatter {
    /// fd = 1 代表标准输出端口
    /// fd = 2 代表错误输出端口
    /// 每次`print`系列宏对应一次`Formatter::new`接口, 会对应多次的`write_***`系列接口.
    /// 可以在这里实现多线程同步机制，避免多线程输出时信息混杂在一起.
    fn new(fd: i32) -> Self
    where
        Self: Sized;
    fn write_buf(&mut self, buf: &[u8]) -> usize;
    fn write_u64(&mut self, val: u64) -> usize {
        self.write_buf(unsafe { u64_buf(val, &mut [0_u8; 24]) })
    }
    fn write_i64(&mut self, val: i64) -> usize {
        self.write_buf(i64_buf(val, &mut [0_u8; 24]))
    }
    fn write_hex(&mut self, val: u64) -> usize {
        self.write_buf(hex_buf(val, &mut [0_u8; 24]))
    }
    fn write_ptr(&mut self, val: *const u8) -> usize {
        self.write_buf(ptr_buf(val, &mut [0_u8; 24]))
    }
    fn write_f64(&mut self, val: f64) -> usize {
        self.write_buf(f64_buf(val, &mut [0_u8; 24]))
    }
    /// # Safety
    /// 调用者保证是一个空指针或者有效的c字符串
    #[inline(never)]
    unsafe fn write_cstr(&mut self, val: *const u8) -> usize {
        if val.is_null() {
            self.write_buf(b"null")
        } else {
            let mut p = val;
            while p.read() != 0 {
                p = p.add(1);
            }
            self.write_buf(core::slice::from_raw_parts(
                val,
                p.offset_from(val) as usize,
            ))
        }
    }
}

#[inline(never)]
fn i64_buf(val: i64, buf: &mut [u8; 24]) -> &[u8] {
    let mut len = unsafe { u64_buf(val.abs() as u64, buf).len() };
    if val < 0 {
        buf[buf.len() - len - 1] = b'-';
        len += 1;
    }
    &buf[buf.len() - len..]
}

#[inline(never)]
fn ptr_buf(val: *const u8, buf: &mut [u8; 24]) -> &[u8] {
    let mut len = hex_buf(val as u64, buf).len();
    buf[buf.len() - len - 1] = b'x';
    buf[buf.len() - len - 2] = b'0';
    len += 2;
    &buf[buf.len() - len..]
}

// # Safety 输入保证buf空间足够
#[inline(never)]
unsafe fn u64_buf(mut val: u64, buf: &mut [u8]) -> &[u8] {
    let mut pos = buf.len();
    loop {
        pos -= 1;
        let n = val % 10;
        buf[pos] = b'0' + (n as u8);
        val /= 10;
        if val == 0 {
            break;
        }
    }
    &buf[pos..]
}

#[inline(never)]
fn hex_buf(mut val: u64, buf: &mut [u8; 24]) -> &[u8] {
    let mut pos = buf.len();
    loop {
        pos -= 1;
        let n = val & 0xF;
        if n < 10 {
            buf[pos] = b'0' + (n as u8);
        } else {
            buf[pos] = b'a' + (n as u8 - 10);
        }
        val >>= 4;
        if val == 0 {
            break;
        }
    }
    &buf[pos..]
}

/// f64::log10/powf依赖std,无法使用. 这里输出的是(+/-)(1/0).dddddd*2^(+/-)d,
/// 是2的指数，而非10的指数
#[inline(never)]
fn f64_buf(val: f64, buf: &mut [u8; 24]) -> &[u8] {
    if val.is_nan() {
        return b"nan";
    };

    if val.is_infinite() {
        if val.is_sign_negative() {
            return b"-inf";
        }
        return b"info";
    }

    let (sign, denormal, fract, exp) = f64_decode(val);
    let mut len = unsafe { u64_buf(exp.abs() as u64, buf).len() };
    len += 1;
    if exp < 0 {
        buf[buf.len() - len] = b'-';
    } else {
        buf[buf.len() - len] = b'+';
    }
    for b in b"^2*" {
        len += 1;
        buf[buf.len() - len] = *b;
    }

    let end = buf.len() - len;
    let d6 = (fract * 1_000_000.0) as u64;
    let len6 = unsafe { u64_buf(d6, &mut buf[..end]).len() };
    len += len6;
    for _ in len6..6 {
        len += 1;
        buf[buf.len() - len] = b'0';
    }

    len += 1;
    buf[buf.len() - len] = b'.';
    len += 1;
    if denormal {
        buf[buf.len() - len] = b'0';
    } else {
        buf[buf.len() - len] = b'1';
    }

    if sign {
        len += 1;
        buf[buf.len() - len] = b'-';
    }

    &buf[buf.len() - len..]
}

// return (sign, denormal, fract, exp)
fn f64_decode(val: f64) -> (bool, bool, f64, i64) {
    let bits = val.to_bits();
    let s = bits >> 63;
    let e = (bits >> 52) & 0x7FF;
    let mut m = bits & ((0x01 << 52) - 1);
    let mut exp = e as i64 - 1023;
    if e == 0 {
        if m == 0 {
            return (true, true, 0.0, 0);
        }
        let h = hi_bit_1(m) as i64;
        exp = -(1022 + 52 - h);
        m <<= 52 - h;
    }

    let mut fract = 0.0f64;
    let mut n = 0.5f64;
    m <<= 12;
    while m > 0 {
        if (m & (0x01 << 63)) > 0 {
            fract += n;
        }
        m <<= 1;
        n /= 2.0;
    }

    (s > 0, e == 0, fract, exp)
}

fn hi_bit_1(mut n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    let mut b = 1;
    let mut m = n & 0xFFFF_FFFF_0000_0000;
    if m > 0 {
        n = m;
        b += 32;
    }
    m = n & 0xFFFF_0000_FFFF_0000;
    if m > 0 {
        n = m;
        b += 16;
    }
    m = n & 0xFF00_FF00_FF00_FF00;
    if m > 0 {
        n = m;
        b += 8;
    }
    m = n & 0xF0F0_F0F0_F0F0_F0F0;
    if m > 0 {
        n = m;
        b += 4;
    }
    m = n & 0xCCCC_CCCC_CCCC_CCCC;
    if m > 0 {
        n = m;
        b += 2;
    }
    m = n & 0xAAAA_AAAA_AAAA_AAAA;
    if m > 0 {
        b += 1;
    }
    b
}

pub struct BufFormatter<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl BufFormatter<'_> {
    /// # Safety
    /// 调用者保证buf指针有效，长度至少为len
    pub unsafe fn new(buf: *mut u8, len: usize) -> Self {
        Self {
            buf: core::slice::from_raw_parts_mut(buf, len),
            pos: 0,
        }
    }
}

impl Formatter for BufFormatter<'_> {
    fn write_buf(&mut self, buf: &[u8]) -> usize {
        let len = buf.len().min(self.buf.len() - self.pos);
        self.buf[self.pos..self.pos + len].copy_from_slice(&buf[..len]);
        self.pos += len;
        buf.len()
    }
    fn new(_fd: i32) -> Self {
        Self {
            buf: &mut [],
            pos: 0,
        }
    }
}

pub use hifmt_macros::nolibc_bprint as bprint;
pub use hifmt_macros::nolibc_cbprint as cbprint;
pub use hifmt_macros::nolibc_ceprint as ceprint;
pub use hifmt_macros::nolibc_ceprintln as ceprintln;
pub use hifmt_macros::nolibc_cprint as cprint;
pub use hifmt_macros::nolibc_cprintln as cprintln;
pub use hifmt_macros::nolibc_csprint as csprint;
pub use hifmt_macros::nolibc_eprint as eprint;
pub use hifmt_macros::nolibc_eprintln as eprintln;
pub use hifmt_macros::nolibc_print as print;
pub use hifmt_macros::nolibc_println as println;
pub use hifmt_macros::nolibc_sprint as sprint;

#[cfg(test)]
mod test {
    use super::*;
    extern crate std;
    use std::*;

    fn datas() -> &'static [f64] {
        &[
            -2.25123f64,
            7.534678f64,
            -2.2578888e-5f64,
            7.5000789e10f64,
            -7.534000123e200f64,
            10.300099911e256,
            10.3000789e-100,
            -0.0,
            f64::MIN,
            f64::MAX,
        ]
    }

    #[test]
    fn test_f64() {
        let f = 1.5f64;
        let (sign, denormal, fract, exp) = f64_decode(f);
        assert_eq!(fract, 0.5f64);
        assert_eq!(exp, 0);
        assert_eq!(sign, false);
        assert_eq!(denormal, false);

        let mut buf = [0_u8; 24];
        for f in datas() {
            let s = f64_buf(*f, &mut buf);
            let s = core::str::from_utf8(s).unwrap();
            let (sign, denormal, fract, exp) = f64_decode(*f);
            let mut nf = if denormal { fract } else { 1.0 + fract };
            nf = nf * 2_f64.powf(exp as f64);
            if sign {
                nf = -nf;
            }
            std::println!("{f}, {nf} {s}");
            assert_eq!(*f, nf);
        }
    }
}
