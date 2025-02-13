# hifmt - 零代码段实现RUST的格式化输出功能

Rename `orion_cfmt` to `hifmt`.

hifmt应用于RUST/C并存的极端受限的嵌入式环境，完全避免使用rust格式化输出功能，将RUST的格式化输出转换为C的格式化输出，最终的目标是是减少二进制大小。

## 版本变更说明

### v0.1.6,v0.1.7

修改代码仓地址，生成feature="nolibc"的文档.

### v0.1.5 增加feature = "nolibc"

实际应用中存在无libc环境，此时没有`dprintf`和`snprintf`的c函数, 可能只是非常简单的字符串输出接口或者纯Rust实现.

最简单情况下用户需要实现一个字符输出接口`Fn(&[u8]) -> usize`:

```
fn write_buf(info: &[u8]) -> usize {
    // ...
    info.len()
}

// 利用make_nolibc_formatter宏将此函数和print系列输出接口关联起来.
hifmt::make_nolibc_formatter!(write_buf);
// 打印输出.
hifmt::print("hello {:rs}", "world");
```

如果用户场景还存在多线程并发打印输出, 需要完整实现`hifmt::Formatter`接口:

```
struct Printer { 
    // ...
};
impl Drop for Printer {
    fn drop(&mut self) {
        // unlock
    }
}

impl hifmt::Formatter for Printer {
    fn new(fd: i32) -> Self {
        Printer {
            // Lock
        }
    }
    fn write_buf(&mut self, info: &[u8]) -> usize {
        // ...
        // info.len()
    }
}

// 利用make_nolibc_formatter宏将此结构体实现和print系列输出接口关联起来.
hifmt::nolibc_formatter!(Printer);
// 打印输出.
hifmt::print("hello: {:rs}", "world");
```

**注意**: 因为`f64::log10`, `f64::powf`依赖`std`, 无法在`no_std`环境使用，因此浮点数的输出格式和`c`语言中的`%e`不同，最终格式为`d.dddd*2^d`用`2`的指数来表达. 使用者可按需替换掉`hifmt::Formatter`中的缺省实现.

## 使用方式Usage

格式化字符串的规则定义如下：

```text
format-spec = {:d|u|x|p|e|cs|rs|rb|cc|rc}
d: 参数类型为整数，按10进制输出，对应%lld
u: 参数类型为整数，按10进制输出，对应%llu
x: 参数类型为整数，按16进制输出，a/b/c/d/e/f, 对应%llx
p: 参数类型为指针，对应%p
e: 参数类型为浮点数, 对应%e
cs: 参数类型为C字符串指针，对应%s
rs: 参数类型为&str, 对应%.*s
rb: 参数类型为&[u8], 对应%.*s
cc: 参数类型为ascii字符，实际转换为c的int类型，对应%c
rc: 参数类型为RUST的char，unicode scalar value，对应%.*s
```

转换后的C函数定为`dprintf(int fd, const char* format, ...)`, 这个函数需要在用户的代码中实现。第一个参数fd，1对应stdout，2对应stderr。
或`snprintf(char* buf, int len, const char* format, ...)`; 

宏的返回值同`dprintf`和`snprintf`的返回值.

hifmt提供如下几个宏：
```rust
//输出到stdout, 转换为dprintf(1, format, ...)
cprint!(format: &'static str, ...);
print!(format: &'static str, ...);

//相对cprint!自动添加\n, 转换为dprintf(1, format "\n", ...)
cprintln!(format: &'static str, ...);
println!(format: &'static str, ...);

//输出到stderr, 转换为dprintf(2, format, ...)
ceprint!(format: &'static str, ...);
eprint!(format: &'static str, ...);

//相对ceprint!自动添加\n, 转换为dprintf(2, format "\n", ...)
ceprintln!(format: &'static str, ...);
eprintln!(format: &'static str, ...);

//输出到buf, 转换为snprintf(buf.as_byte().as_ptr(), buf.len(), format, ...)
csprint!(buf: &mut str, format: &'static str, ...)
sprint!(buf: &mut str, format: &'static str, ...)

//输出到buf, 转换为snprintf(buf.as_ptr(), buf.len(), format, ...)
cbprint!(buf: &mut [u8], format: &'static str, ...)
bprint!(buf: &mut [u8], format: &'static str, ...)


```

在RUST中的使用方法如下：

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

cargo expand的代码如下：

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

## 方案分析

RUST/C混合应用场景，RUST的格式化输出无条件转化为C的格式化输出接口，同时还不能完全消除对Display/Debug trait的依赖，这样RUST的格式化输出的开销可以全部消除，实现方案可以做到空间最优。

最理想的情况是，格式化输出的格式完全遵循RUST的定义如下所示：

```rust
fn main() {
	let str = "sample";
	cprintln!("cprintln hex = {:x} digital = {} str = {}", 100_i32, 99_i64, str);
}
```

经过过程宏cprintln处理之后，能变换为下面的代码：

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

要实现上面的转化，对过程宏有几个要求：
1. RUST的字符串需要转化为C的字符串，必须增加\0结束符。
2. RUST的过程宏需要识别出参数的类型宽度，不同宽度对应不同的C格式化字符，比如%d还是%lld
3. RUST的过程宏还需要识别出参数的类型，如果是字符串，格式化字符中需要指定长度信息，%.\*s, 字符参数也需要一分为二，分别传递长度和\*const u8的指针。

很遗憾，RUST过程宏并非无所不能。过程宏工作的时候，类型的命名解析还未最终完成，意味着对于下面的场景，无法识别出这是一个i32类型：

```rust
type my_i32 = i32;
let i: my_i32 = 100;
cprintln!("i32 = {}", i);
```

过程宏中，最多知道i的类型是my_i32，并不知道my_i32和i32是等价的。实际场景还会更复杂，参数可能是一个变量，也可能是一个函数调用的返回值。因此期望过程宏能够识别出具体参数的类型，基于类型做转化处理是无法实现的。

在RUST的原生实现中，定义Display/Debug Trait，也就解决了类型问题，即将任何类型都归一到Display/Debug Trait，基于这个trait的接口进行转化处理。

我们的目标是期望彻底消除Display/Debug Trait的代码，不能依赖一个统一的Trait定义，那么只能同C的思路，通过格式化字符来区分参数类型。rust的原生实现中也是通过特殊的格式化字符'?'来区分是基于Display Trait还是Debug Trait的接口实现输出，一样的原理。如下所示：

```rust
fn main() {
	cprintln!("cprintln hex = {:x} digital = {:lld} str = {:s}", 100_i32, 99_i64, str);
}
```

过程宏中基于格式化字符来实现对应参数的转化处理是可行的。不过上面这种方式有个问题，就是格式化字符中同时也指定了参数宽度信息，比如{:x}对应C的int类型，而{:lld}对应C的long long int类型，这种方式要求使用者必须保证格式化字符和参数的宽度必须一致，如果不一致可能导致非法地址访问等错误，降低了代码的安全性。考虑到这一点，做一个简化，格式化字符中只定义数据类型，不定义数据宽度，实际上是统一数据类型为C的long long int和double类型。如下所示：

```rust
fn main() {
	cprintln!("cprintln hex = {:x} digital = {:d} str = {:s}", 100_i32, 99_i64, str);
}
```

过程宏转化后的代码为：

```rust
fn main() {
	unsafe {
		printf("cprintln hex = %llx digital = %lld str = %.*s\n\0".as_bytes().as_ptr(), 100_i32 as i64, 99_i64 as i64, str.len() as i32, str.as_bytes().as_ptr());
	}
}
```

通过这种方式，RUST代码的安全性大大提高，如果参数类型传递错误，编译就会失败，不会隐藏问题。

### 字符串的特殊之处

对于字符串，因为需要传递长度，从RUST的一个参数，转换为2个参数，这里有一个副作用。如下所示：

```rust
cprintln!("str = {:s}", get_str());
```

转换后的代码如下：

```rust
unsafe {
	printf("str = %.*s\n\0".as_bytes().as_ptr(), get_str().len(), get_str().as_bytes().as_ptr());
}
```

注意到get_str()被调用了2次，这和C的宏可能出现的副作用类似，多次调用对系统的影响是未知的，需要避免。

简单的情况，作为使用注意事项，要求RUST程序员保证对于字符串的格式化输出，不能传递返回字符串的函数调用，而必须是一个变量，这降低了易用性。

最好的方式是过程宏中 能够判断出参数是否是函数调用，如果是，则自动生成一个临时变量。也可以无条件的将所有字符串参数定义为临时变量，这样如果传递的字符串类型如果不是&str则会报告明确的错误信息。

### rust char的特殊处理

rust char是unicode编码，格式化输出需要基于char::encode_utf8转换为作为字符串输出，但是char::encode_utf8的调用自动引入core::fmt相关的很多符号，导致二进制大小增大。

为了消除自动引入的core::fmt相关代码，对于rust char需要自己实现转换功能，如下所示是第一个版本：

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

上面的实现，虽然没有显示引入任何core::fmt相关的代码，但生成之后的二进制，仍然会引入core::fmt相关的代码：

```bash
h00339793@DESKTOP-MOPEH6E:~/working/rust/orion/main$ nm target/debug/orion | grep fmt
0000000000002450 T _ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$u64$GT$3fmt17h7afd8f52b570e595E
0000000000002450 T _ZN4core3fmt3num3imp54_$LT$impl$u20$core..fmt..Display$u20$for$u20$usize$GT$3fmt17h95817e498b69c414E
0000000000001d70 t _ZN4core3fmt9Formatter12pad_integral12write_prefix17h9921eded510830d2E
00000000000018f0 T _ZN4core3fmt9Formatter12pad_integral17hd85ab5f2d47ca89bE
0000000000001020 T _ZN4core9panicking9panic_fmt17h940cb25cf018faefE
h00339793@DESKTOP-MOPEH6E:~/working/rust/orion/main$
```

这些符号应该是因为通过动态下标访问数组可能越界，而rust在越界处理过程中会引入core::fmt相关的代码。为了消除这部分代码空间，必须全部消除动态数组下标访问，最终确定的实现版本如下：

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

这也提醒我们，业务层代码需要**避免动态下标访问数组元素**, 避免自动引入core::fmt相关的代码。
