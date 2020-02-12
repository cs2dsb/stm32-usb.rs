#![recursion_limit = "128"]

extern crate proc_macro;
use self::proc_macro::TokenStream;

use quote::{quote, format_ident};
use syn::parse::{Parse, ParseStream, Result as SynResult};
use syn::{self, Ident, Token, Visibility, LitStr, Error};

use std::fs::File;
use std::path::PathBuf;
use std::io::{BufReader, BufRead};
use std::fmt::Display;
use std::num::ParseIntError;

use proc_macro2::Span;

/*
asc_list_to_enum! {
    pub MyEnum = "...path...";
    $VISIBILITY $NAME = $PATH;
}
*/

struct AscEnum {
    visibility: Visibility,
    name: Ident,
    path: LitStr,
}

impl Parse for AscEnum {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let visibility: Visibility = input.parse()?;
        let name: Ident = input.parse()?;
        input.parse::<Token![=]>()?;
        let path: LitStr = input.parse()?;
        input.parse::<Token![;]>()?;
        Ok(AscEnum {
            visibility,
            name,
            path,
        })
    }
}

#[proc_macro]
pub fn asc_list_to_enum(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    match impl_asc_list_to_enum(&ast) {
        Ok(ts) => ts.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn wrap_err<T: Display>(span: Span, message: T) -> Error {
    Error::new(span, message)
}

fn impl_asc_list_to_enum(input: &AscEnum) -> SynResult<impl Into<TokenStream>> {
    let dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let path = PathBuf::from(input.path.value());
    let target = if path.is_relative() {
        dir.join(path)
    } else {
        path
    };

    let path_span: Span = input.path.span().unwrap().into();

    let file = match File::open(target) {
        Ok(f) => f,
        Err(e) => Err(wrap_err(path_span, &e))?,
    };
    let reader = BufReader::new(file);

    let mut entries = Vec::new();

    let mut found_start = false;
    for (i, line) in reader.lines().enumerate() {
        let line = match line {
            Err(e) => Err(wrap_err(path_span, &e))?,
            Ok(l) => l,
        };
        let line_num = i + 1;

        if found_start {
            match process_line(&line, &mut entries) {
                Err(e) => Err(wrap_err(path_span, 
                    format!("Error on line {}: {}", line_num, e)))?,
                Ok(_) => {},
            }
        } else {
            if line.starts_with("--------") {
                found_start = true;
            }
        }
    }

    let AscEnum {
        visibility,
        name,
        ..
    } = input;

    let quoted_entries = entries.iter().map(|e| {
        let name = format_ident!("{}", e.name);
        let comment = &e.comment;
        quote! {
            #[doc = #comment]
            #name
        }
    });

    let quoted_asc = entries.iter().map(|e| {
        let variant_name = format_ident!("{}", e.name);
        let value = e.asc;
        quote! {            
            #name::#variant_name => #value
        }
    });

    let quoted_ascq = entries.iter().map(|e| {
        let variant_name = format_ident!("{}", e.name);
        let value = e.ascq;
        quote! {            
            #name::#variant_name => #value
        }
    });

    let quoted_from = entries.iter().map(|e| {
        let variant_name = format_ident!("{}", e.name);
        let asc = e.asc;
        let ascq = e.ascq;
        quote! {            
            (#asc, #ascq) => Some(#name::#variant_name)
        }
    });

    let quoted_from_str = entries.iter().map(|e| {
        let variant_name_ident = format_ident!("{}", e.name);
        let variant_name = &e.name;
        quote! {            
            #variant_name => Some(#name::#variant_name_ident)
        }
    });

    let quoted_from_str_lower = entries.iter().map(|e| {
        let variant_name_ident = format_ident!("{}", e.name);
        let variant_name = &e.name.to_lowercase().to_string();
        quote! {            
            #variant_name => Some(#name::#variant_name_ident)
        }
    });

    let result = quote! {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        #visibility enum #name {
            #( #quoted_entries ),
            *
        }
        impl #name {
            /// Returns the ASC code for this variant
            pub fn asc(&self) -> u8 {
                match self {
                    #( #quoted_asc ),
                    *
                }
            }
            /// Returns the ASCQ code for this variant
            pub fn ascq(&self) -> u8 {
                match self {
                    #( #quoted_ascq ),
                    *
                }
            }

            /// Returns the ASCQ code for this variant
            pub fn from(asc: u8, ascq: u8) -> core::option::Option<Self> {
                match (asc, ascq) {
                    #( #quoted_from ),
                    *,
                    _ => None,
                }
            }
        }
        impl packed_struct::PrimitiveEnum for #name {
            type Primitive = u16;
            fn from_primitive(val: Self::Primitive) -> Option<Self> {
                let asc = ((val >> 8) & 0xFF) as u8;
                let ascq = ((val >> 0) & 0xFF) as u8;
                Self::from(asc, ascq)
            }
            fn to_primitive(&self) -> Self::Primitive {
                (self.asc() as u16) << 8 | (self.ascq() as u16)
            }
            fn from_str(s: &str) -> Option<Self> {
                match s {
                    #( #quoted_from_str ),
                    *,
                    _ => None,
                }
            }
            fn from_str_lower(s: &str) -> Option<Self> {
                match s {
                    #( #quoted_from_str_lower ),
                    *,
                    _ => None,
                }
            }
        }
    };
//panic!("{:?}", result.to_string());
    Ok(result)
}

struct Code {
    asc: u8,
    ascq: u8,
    name: String,
    comment: String,
}

fn process_line(line: &str, entries: &mut Vec<Code>) -> Result<(), ParseIntError> {
    if line.len() < 25 {
        // A reserved code that has no description yet
        return Ok(());
    }

    let asc = &line[0..2];
    let ascq = &line[4..6];

    let asc = u8::from_str_radix(asc, 16)?;

    let description = &line[24..];
    if description.starts_with("Obsolete") {
        return Ok(());
    }

    let mut name = String::new();
    let mut uppercase = true;

    for c in description.chars() {
        if c == '(' {
            break;
        }
        if !c.is_alphabetic() {
            uppercase = true;
            continue;
        }
        if uppercase {
            name += &c.to_uppercase().to_string();
            uppercase = false;
        } else {
            name += &c.to_lowercase().to_string();
        }
    }

    let mut add = |mut code: Code| {
        let mut found = true;
        let mut n = 0;
        let mut new_name = String::new(); 
        while found {
            new_name = if n == 0 {
                code.name.clone()
            } else {
                format!("{}_{}", code.name, n)
            };
            found = false;
            for o in entries.iter() {
                if o.name == new_name {
                    found = true;
                    break;
                }
            }
            n += 1;
        }
        if new_name != code.name {
            code.name = new_name;
        }
        entries.push(code);
    };
    
    if ascq == "NN" && asc == 0x40 {
        for ascq in 0x80..=0xFF {
            add(Code {
                asc,
                ascq,
                name: format!("DiagnosticFailureOnComponent_{:X?}", ascq),
                comment: format!("ASC 0x{:X?}, ASCQ/COMPONENT: 0x{:X?} - {}", asc, ascq, description),
            });
        }
    } else if ascq == "NN" && asc == 0x4D {
        for ascq in 0x00..=0xFF {
            add(Code {
                asc,
                ascq,
                name: format!("{}_0x{:X?}", name, ascq),
                comment: format!("ASC 0x{:X?}, ASCQ/TASK TAG: 0x{:X?} - {}", asc, ascq, description),
            });
        }
    } else if ascq == "NN" && asc == 0x70 {
        for ascq in 0x00..=0xFF {
            add(Code {
                asc,
                ascq,
                name: format!("DecompressionExceptionShortAlgorithmIdOf_{:X?}", ascq),
                comment: format!("ASC 0x{:X?}, ASCQ/ANLGORITHM ID: 0x{:X?} - {}", asc, ascq, description),
            });
        }
    } else {
        let ascq = u8::from_str_radix(ascq, 16)?;
        add(Code {
            asc,
            ascq,
            name,
            comment: format!("ASC 0x{:X?}, ASCQ: 0x{:X?} - {}", asc, ascq, description),
        });
    }

    Ok(())
}