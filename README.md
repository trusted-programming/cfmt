# hifmt - Format output without Rust code segment in binary to reduce the ultimate binary size

Rename `orion_cfmt` to `hifmt`.

Restricted on embedded systems, the goal of `hifmt` is to reduce the ultimate
binary size. With `hifmt`, one could avoid the uses of Rust format print
function by converting them into formatted print in C. 

## Usage

The specification of the formatted strings is defined as follows:

```text
format-spec = {:d|u|x|p|e|cs|rs|rb|cc|rc}
d: print int as digits, see %lld
u: print int as hexdecimals, see %llu
x: print int as hexdecimals a/b/c/d/e/f, see %llx
p: print pointer，see %p
e: print floating point numbers, see %e
cs: print C string pointers, see %s
rs: print Rust string &str, see %.*s
rb: print Rust slice &[u8], see %.*s
cc: print ASCII char into int type in C, see %c
rc: print Rust char into unicode scalar value, see %s
```
The converted C function is defined as `dprintf(int fd, const char* format, ...)`, which needs to be implemented in the user's code. The first parameter is fd. The value 1 indicates stdout, and the value 2 indicates stderr. or `snprintf(char* buf, int len, const char* format, . . . ) `;

The return value of the macro is the same as that of'dprintf' and'snprintf'.

`hifmt` provides the following macros:
```rust
// print to stdout, converted into dprintf(1, format, ...)
cprint!(format: &'static str, ...);
print!(format: &'static str, ...);

// append \n to cprint!, converted into dprintf(1, format "\n", ...)
cprintln!(format: &'static str, ...);
println!(format: &'static str, ...);

// print to stderr, converted into dprintf(2, format, ...)
ceprint!(format: &'static str, ...);
eprint!(format: &'static str, ...);

// append \n to ceprint!, converted into dprintf(2, format "\n", ...)
ceprintln!(format: &'static str, ...);
eprintln!(format: &'static str, ...);

//write to buf, converted into snprintf(buf.as_byte().as_ptr(), buf.len(), format, ...)
csprint!(buf: &mut str, format: &'static str, ...)
sprint!(buf: &mut str, format: &'static str, ...)

//write to buf, converted into snprintf(buf.as_ptr(), buf.len(), format, ...)
cbprint!(buf: &mut [u8], format: &'static str, ...)
bprint!(buf: &mut [u8], format: &'static str, ...)
```

The usage in Rust is shown as follows:

```rust
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
```

After cargo expand, the above code becomes:

```rust
#[link(name = "c")]
extern "C" {
    fn dprintf(fd: i32, format: *const u8, ...) -> i32;
    fn snprintf(buf: *mut u8, len: usize, format: *const u8, ...) -> i32;
}
fn main() {
    let s = ::alloc::vec::from_elem(b'\0', 100);
    let s = &mut String::from_utf8(s).unwrap();
    {
        {
            let _hifmt_1: &str = "hello snprintf";
        }
        let _hifmt_0: &mut str = s;
        let _hifmt_1: &str = "hello snprintf";
        unsafe {
            snprintf(
                _hifmt_0.as_bytes_mut().as_mut_ptr(),
                _hifmt_0.len() as usize,
                "sprint(%.*s)\0".as_bytes().as_ptr(),
                _hifmt_1.len() as i32,
                _hifmt_1.as_bytes().as_ptr(),
            );
        }
    };
    let b = &mut [0_u8; 100];
    {
        {
            let _hifmt_1: &str = "hello snprintf";
        }
        let _hifmt_0: &mut [u8] = b;
        let _hifmt_1: &str = "hello snprintf";
        unsafe {
            snprintf(
                _hifmt_0.as_mut_ptr(),
                _hifmt_0.len() as usize,
                "bprint(%.*s)\0".as_bytes().as_ptr(),
                _hifmt_1.len() as i32,
                _hifmt_1.as_bytes().as_ptr(),
            );
        }
    };
    {
        {
            let _hifmt_1 = (100) as i64;
        }
        {
            let _hifmt_2 = (200) as i64;
        }
        {
            let _hifmt_3 = (300) as i64;
        }
        {
            let _hifmt_4 = (400.0) as f64;
        }
        {
            let _hifmt_5 = (b) as *const _ as *const u8;
        }
        {
            let _hifmt_6 = (b) as *const _ as *const u8;
        }
        {
            let _hifmt_7: &str = s;
        }
        {
            let _hifmt_8: &[u8] = b;
        }
        let _hifmt_1 = (100) as i64;
        let _hifmt_2 = (200) as i64;
        let _hifmt_3 = (300) as i64;
        let _hifmt_4 = (400.0) as f64;
        let _hifmt_5 = (b) as *const _ as *const u8;
        let _hifmt_6 = (b) as *const _ as *const u8;
        let _hifmt_7: &str = s;
        let _hifmt_8: &[u8] = b;
        unsafe {
            dprintf(
                1i32,
                "d = %lld u = %llu x = %llx e = %e p = %p cstr = %s str = %.*s bytes = %.*s\n\0"
                    .as_bytes()
                    .as_ptr(),
                _hifmt_1,
                _hifmt_2,
                _hifmt_3,
                _hifmt_4,
                _hifmt_5,
                _hifmt_6,
                _hifmt_7.len() as i32,
                _hifmt_7.as_bytes().as_ptr(),
                _hifmt_8.len() as i32,
                _hifmt_8.as_ptr(),
            );
        }
    };
}
```

## Design Rationale

While mixing Rust/C, unconditionally convert Rust's formated prints into C's API could completely remove the dependencies on
Display/Debug traits, thereby eliminating the overhead of Rust formatted printing and achieving the optimal size.

Ideally, the formatted print follows the spec in Rust as follows:

```rust
fn main() {
	let str = "sample";
	cprintln!("cprintln hex = {:x} digital = {} str = {}", 100_i32, 99_i64, str);
}
```

After expanding with the proc macro `cprintln!`, it becomes

```rust
#[link(name = "c")]
extern "C" {
	fn printf(format: *const u8, ...) -> i32
}

fn main() {
	let str = "sample";
	unsafe {
		printf("cprintln hex = %x digital = %lld str = %.*s\n\0".as_bytes().as_ptr(), 100_i32, 99_i64, str.len() as i32, str.as_bytes().as_ptr());
	}
}
```

To implement the above, we need to have the proc macro satisfy the following requirements:
1. RUST strings need to be ended with \0 in C;
2. RUST argument size needs to be recognized by the proc macro so as to determine which C format to use, e.g., whether it is `%d` or `%lld`;
3. RUST argument type needs to be recognized by the proc macro: the format needs to specify the length if it is a string, and
   separately treating char arguments with an `*const u8` pointer with length。

Unfortunately, proc macros cannot achieve all that. When the are expanded, the parsing has not been done to determine the variable's types.
For example, the i32 type in the following code:

```rust
type my_i32 = i32;
let i: my_i32 = 100;
cprintln!("i32 = {}", i);
```

At best, the proc macro can tell the type of `i` is `my_i32`, without knowing that actually `my_i32` is equivalent to `i32`.

In fact, in more complex scenarios, the arguments could be variables, or the value returned from a function call. Therefore, 
it is unrealistic to expect that the proc macro could recognize the type of certain arguments, making it impossible to realize the above ideal
solution.

The current implementation of Rust defines `Display/Debug` traits in response to the type problem by unifying all types into
Display/Debug trait, and perform the conversion based on the interfaces of such traits.

Our objective is to further eliminate the needs of `Display/Debug` traits, so we have to determine argument types based on the format string.
In fact, Rust also use special characters such as '?' to determine whether a Display or a Debug trait is to be used. o
Following the same principle, we could leverage on the format strings as follows:

```rust
fn main() {
	cprintln!("cprintln hex = {:x} digital = {:lld} str = {:s}", 100_i32, 99_i64, str);
}
```

This makes it feasible to rely on proc macros. However, there is a problem in
the above, that is, the format string also restricts the argument sizes. For
example, `{:x}` is `int`, while `{:lld}` is `long long int` in C. It requires
the programmer to guarantee the consistency between the format string and the
argument size. Otherwise, invalid address access could lower the safety of
code. In this regard, we need to provide a simplification, whereby the format
string only defines data type, whitout specifying data size, which in effect
unify the data types into `long long int` or `double` in C.

```rust
fn main() {
	cprintln!("cprintln hex = {:x} digital = {:d} str = {:s}", 100_i32, 99_i64, str);
}
```

As a result, the proc macro generates the following code:

```rust
fn main() {
	unsafe {
		printf("cprintln hex = %llx digital = %lld str = %.*s\n\0".as_bytes().as_ptr(), 100_i32 as i64, 99_i64 as i64, str.len() as i32, str.as_bytes().as_ptr());
	}
}
```

As such, the safety of Rust code could be ensured: if a wrong argument type is passed on, the compiler would reject it rather than hiding the problem.

### Special treatment of string

For strings, the length information has to be passed on, therefore an argument in Rust will to converted into two, causing some side effect.
This is illustrated below:

```rust
cprintln!("str = {:s}", get_str());
```

The generated code reads as follows:

```rust
unsafe {
	printf("str = %.*s\n\0".as_bytes().as_ptr(), get_str().len(), get_str().as_bytes().as_ptr());
}
```

Note that `get_str()` has been invoked twice, which is like side effect in macro of C where the effect is unknown when the macro
is to be expanded more than once. This problem needs to be avoided.

In simple terms, the programmer needs to guarantee the string format output cannot pass function calls that return strings instead of
variables. That would reduce usability.

A best choice is to judge whether the argument is a function call. If so, generate a temporary variable. Alternatively, define every string argument
as a temporary variable unconditionally, and report errors explicitly when the string argument is not a `&str`. 

### Special treatment of Rust char

Rust char is encoded in unicode, its format output needs to be based on `char::encode_utf8` to convert into strings; however,
the use of `char::encode_utf8` would automatically introduce symbols of `core::fmt`, causing the bloat of binary size.

To avoid introducing `core::fmt` crate, we need to implement the conversion for Rust `char`. 
The first version of implementation is shown below:

```rust
pub fn encode_utf8(c: char, buf: &mut [u8]) -> &[u8] {
    let mut u = c as u32;
    let bits: &[u32] = &[0x7F, 0x1F, 0xF, 0xFFFFFFFF, 0x00, 0xC0, 0xE0, 0xF0];
    for i in 0..buf.len() {
        let pos = buf.len() - i - 1;
        if u <= bits[i] {
            buf[pos] = (u | bits[i + 4]) as u8;
            unsafe { return core::slice::from_raw_parts(&buf[pos] as *const u8, i + 1); }
        }
        buf[pos] = (u as u8 & 0x3F) | 0x80;
        u >>= 6;
    }
    return &buf[0..0];
}
```

Although nothing is done explicitly, the binary still include related symbols in `core::fmt`:

```bash
h00339793@DESKTOP-MOPEH6E:~/working/rust/orion/main$ nm target/debug/orion | grep fmt
0000000000002450 T _ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$u64$GT$3fmt17h7afd8f52b570e595E
0000000000002450 T _ZN4core3fmt3num3imp54_$LT$impl$u20$core..fmt..Display$u20$for$u20$usize$GT$3fmt17h95817e498b69c414E
0000000000001d70 t _ZN4core3fmt9Formatter12pad_integral12write_prefix17h9921eded510830d2E
00000000000018f0 T _ZN4core3fmt9Formatter12pad_integral17hd85ab5f2d47ca89bE
0000000000001020 T _ZN4core9panicking9panic_fmt17h940cb25cf018faefE
h00339793@DESKTOP-MOPEH6E:~/working/rust/orion/main$
```

These symbols are added to check the array indices dynamically in Rust to prevent buffer overflow.
To eliminate such code in binary, we need to disable all array index checks, which become the following:

```rust
pub fn encode_utf8(c: char, buf: &mut [u8; 5]) -> *const u8 {
    let mut u = c as u32;
    if u <= 0x7F {
        buf[0] = u as u8;
        return buf as *const u8;
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
```

This reminds us, that use code needs to **avoid the use of dynamic check of array indices**, in order to avoid introducing `core::fmt` dependency.
