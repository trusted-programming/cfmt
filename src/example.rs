#[link(name = "c")]
extern "C" {
    fn dprintf(fd: i32, format: *const u8, ...) -> i32;
    fn snprintf(buf: *mut u8, len: usize, format: *const u8, ...) -> i32;
}
fn main() {
    let s = vec![b'\0'; 100];
    let s = &mut String::from_utf8(s).unwrap();
    hifmt::sprint!(s, "sprint({:rs})", "hello snprintf");

    let b = &mut [0_u8; 100];
    hifmt::bprint!(b, "bprint({:rs})", "hello snprintf");

    hifmt::println!("d = {:d} u = {:u} x = {:x} e = {:e} p = {:p} cstr = {:cs} str = {:rs} bytes = {:rb}",
        100, 200, 300, 400.0, b, b, s, b);
}
