//
// 实现过程参考了UFMT的实现，重用了其部分代码，parse/unescape，涉及到格式化字符串的解析
// 如果后续因为知识产权等原因不能重用，一个字符串解析过程要完全重写并不复杂。
//
extern crate proc_macro;
use core::mem;
use std::borrow::Cow;
use proc_macro2::{ Span };
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input,
    Expr, LitStr, Token,
    parse::{self, Parse, ParseStream },
    punctuated::Punctuated,
    spanned::{ Spanned },
};

#[proc_macro]
pub fn print(input: TokenStream) -> TokenStream {
    cprintf(input, false, 1)
}

#[proc_macro]
pub fn cprint(input: TokenStream) -> TokenStream {
    cprintf(input, false, 1)
}

#[proc_macro]
pub fn println(input: TokenStream) -> TokenStream {
    cprintf(input, true, 1)
}

#[proc_macro]
pub fn cprintln(input: TokenStream) -> TokenStream {
    cprintf(input, true, 1)
}

#[proc_macro]
pub fn eprint(input: TokenStream) -> TokenStream {
    cprintf(input, false, 2)
}

#[proc_macro]
pub fn ceprint(input: TokenStream) -> TokenStream {
    cprintf(input, false, 2)
}

#[proc_macro]
pub fn eprintln(input: TokenStream) -> TokenStream {
    cprintf(input, true, 2)
}

#[proc_macro]
pub fn ceprintln(input: TokenStream) -> TokenStream {
    cprintf(input, true, 2)
}

fn cprintf(input: TokenStream, ln: bool, fd: i32) -> TokenStream {
    let input = parse_macro_input!(input as Input);
    let mut format = input.format.value();
    
    if ln {
        format.push_str("\n\0");
    } else {
        format.push('\0');
    }

    let pieces = match parse(&format, input.format.span()) {
        Err(e) => return e.to_compile_error().into(),
        Ok(pieces) => pieces,
    };

    let argc: usize = input.args.len();
    let required_argc: usize = pieces.iter().filter(|piece| !piece.is_literal()).count();

    if argc != required_argc {
        return parse::Error::new(input.format.span(),
            &format!("format string required {} arguments but {} were supplied",
                required_argc, argc)).to_compile_error().into();
    }

    let literal = gen_literal(&pieces);
    let mut exprs = vec![];
    let mut vars = vec![];

    let mut i = 0;
    for piece in pieces {
        if matches!(piece, Piece::Literal(_)) {
            continue;
        }
        let arg = &input.args[i];
        match piece {
            Piece::Literal(_) => {
            },
            Piece::Str | Piece::Bytes => {
                let name = format!("_cfmt_{}_", i);
                let ident = syn::Ident::new(&name, arg.span());
                exprs.push(quote!(#ident.len() as i32));
                if matches!(piece, Piece::Str) {
                    vars.push(quote!(let #ident: &str = #arg;));
                    exprs.push(quote!(#ident.as_bytes().as_ptr()));
                } else {
                    vars.push(quote!(let #ident: &[u8] = #arg;));
                    exprs.push(quote!(#ident.as_ptr()));
                }
            },
            Piece::Char => {
                let name = format!("_cfmt_{}_", i);
                let ident = syn::Ident::new(&name, arg.span());
                vars.push(quote!(
                    let mut #ident = [0_u8; 5];
                    let #ident = cfmt::encode_utf8(#arg, &mut #ident);
                ));
                exprs.push(quote!(#ident));
            },
            Piece::CChar => {
                exprs.push(quote!(#arg as i32));
            },
            Piece::CStr | Piece::Pointer => {
                exprs.push(quote!(#arg as *const _));
            },
            Piece::Double => {
                exprs.push(quote!(#arg as f64));
            },
            _ => {
                exprs.push(quote!(#arg as i64));
            }
        }
        i += 1;
    }

    let tokens = quote!{ unsafe { #(#vars)* dprintf( #fd, #literal.as_bytes().as_ptr(), #(#exprs),*); } };
    tokens.into()
}

fn gen_literal(pieces: &Vec<Piece>) -> String {
    let mut buf = String::new();
    pieces.iter().all(|piece| {
        match piece {
            Piece::Literal(s) => buf.push_str(&s),
            Piece::CStr => buf.push_str("%s"),
            Piece::Pointer => buf.push_str("%p"),
            Piece::Str => buf.push_str("%.*s"),
            Piece::Bytes => buf.push_str("%.*s"),
            Piece::Signed => buf.push_str("%lld"),
            Piece::Unsigned => buf.push_str("%llu"),
            Piece::Hex => buf.push_str("%llx"),
            Piece::Char => buf.push_str("%s"),
            Piece::CChar => buf.push_str("%c"),
            Piece::Double => buf.push_str("%e"),
        }
        true
    });
    buf
}

struct Input {
    format: LitStr,
    _comma: Option<Token![,]>,
    args:   Punctuated<Expr, Token![,]>, 
}

impl Parse for Input {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let format = input.parse()?;
        if input.is_empty() {
            Ok(Input {
                format: format,
                _comma: None,
                args:   Punctuated::new(),
            })
        } else {
            Ok(Input {
                format: format,
                _comma: input.parse()?,
                args:   Punctuated::parse_terminated(input)?,
            })
        }
    }
}

enum Piece<'a> {
    Literal(Cow<'a, str>),
    CStr,
    Pointer,
    CChar,
    Char,
    Str,
    Bytes,
    Hex,
    Unsigned,
    Signed,
    Double,
}

impl Piece<'_> {
    fn is_literal(&self) -> bool {
        matches!(self, Piece::Literal(_))
    }
}

fn parse(mut format: &str, span: Span) -> parse::Result<Vec<Piece>> {
    let mut pieces = vec![];
    let mut buf = String::new();
    loop {
        let mut parts = format.splitn(2, '{');
        match (parts.next(), parts.next()) {
            (None, None) => break,
            (Some(s), None) => {
                if buf.is_empty() {
                    if !s.is_empty() {
                        pieces.push(Piece::Literal(unescape(s, span)?));
                    }
                } else {
                    buf.push_str(&unescape(s, span)?);
                    pieces.push(Piece::Literal(Cow::Owned(buf)));
                }
                break;
            },
            (head, Some(tail)) => {
                const CSTR: &str = ":cs}";
                const POINTER: &str = ":p}";
                const STR: &str = ":rs}";
                const BYTES: &str = ":rb}";
                const HEX: &str = ":x}";
                const SIGNED: &str = ":d}";
                const UNSIGNED: &str = ":u}";
                const DOUBLE: &str = ":e}";
                const CCHAR: &str = ":cc}";
                const CHAR: &str = ":rc}";
                const ESCAPE_BRACE: &str = "{";

                let head = head.unwrap_or("");
                if tail.starts_with(CSTR)
                    || tail.starts_with(POINTER)
                    || tail.starts_with(STR)
                    || tail.starts_with(BYTES)
                    || tail.starts_with(HEX)
                    || tail.starts_with(SIGNED)
                    || tail.starts_with(UNSIGNED)
                    || tail.starts_with(DOUBLE)
                    || tail.starts_with(CCHAR)
                    || tail.starts_with(CHAR)
                {
                    if buf.is_empty() {
                        if !head.is_empty() {
                            pieces.push(Piece::Literal(unescape(head, span)?));
                        }
                    } else {
                        buf.push_str(&unescape(head, span)?);
                        pieces.push(Piece::Literal(Cow::Owned(mem::take(&mut buf))));
                    }
                    
                    if let Some(tail_tail) = tail.strip_prefix(CSTR) {
                        pieces.push(Piece::CStr);
                        format = tail_tail;
                    } else if let Some(tail_tail) = tail.strip_prefix(POINTER) {
                        pieces.push(Piece::Pointer);
                        format = tail_tail;
                    } else if let Some(tail_tail) = tail.strip_prefix(STR) {
                        pieces.push(Piece::Str);
                        format = tail_tail;
                    } else if let Some(tail_tail) = tail.strip_prefix(BYTES) {
                        pieces.push(Piece::Bytes);
                        format = tail_tail;
                    } else if let Some(tail_tail) = tail.strip_prefix(HEX) {
                        pieces.push(Piece::Hex);
                        format = tail_tail;
                    } else if let Some(tail_tail) = tail.strip_prefix(SIGNED) {
                        pieces.push(Piece::Signed);
                        format = tail_tail;
                    } else if let Some(tail_tail) = tail.strip_prefix(UNSIGNED) {
                        pieces.push(Piece::Unsigned);
                        format = tail_tail;
                    } else if let Some(tail_tail) = tail.strip_prefix(CCHAR) {
                        pieces.push(Piece::CChar);
                        format = tail_tail;
                    } else if let Some(tail_tail) = tail.strip_prefix(CHAR) {
                        pieces.push(Piece::Char);
                        format = tail_tail;
                    } else if let Some(tail_tail) = tail.strip_prefix(DOUBLE) {
                        pieces.push(Piece::Double);
                        format = tail_tail;
                    }
                    
                } else if let Some(tail_tail) = tail.strip_prefix(ESCAPE_BRACE) {
                    buf.push_str(&unescape(head, span)?);
                    buf.push('{');
                    format = tail_tail;
                } else {
                    return Err(parse::Error::new(span,
                        "invalid format string: expected {:d}, {:u}, {:x}, {:e}, {:p}, {:cs}, {:rs}, {:rb} {:cc} {:rc} {{"));
                }
            }
        }
    }

    Ok(pieces)
}

fn unescape(mut format: &str, span: Span) -> parse::Result<Cow<str>> {
    if format.contains('}') {
        let mut buf = String::new();
        while format.contains('}') {
            const ERR: &str = "invalid format string: unmatched right brace";
            let mut parts = format.splitn(2, '}');
            match (parts.next(), parts.next()) {
                (Some(head), Some(tail)) => {
                    const ESCAPE_BRACE: &str = "}";
                    if let Some(tail_tail) = tail.strip_prefix(ESCAPE_BRACE) {
                        buf.push_str(head);
                        buf.push('}');
                        format = tail_tail;
                    } else {
                        return Err(parse::Error::new(span, ERR));
                    }
                },
                _ => unreachable!(),
            }
        }
        buf.push_str(format);
        Ok(buf.into())
    } else {
        Ok(Cow::Borrowed(format))
    }
}

