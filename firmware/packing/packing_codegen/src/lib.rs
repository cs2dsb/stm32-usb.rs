
#![cfg_attr(feature = "diagnostic-notes", feature(proc_macro_diagnostic))]

extern crate proc_macro;
use self::proc_macro::TokenStream;

use quote::{ quote, format_ident };
use syn::{
    spanned::Spanned,
    Ident, 
    Error,
    parse_macro_input, 
    Data,
    DeriveInput, 
    Fields, 
    Attribute, 
    Path, 
    Meta,
    Lit,
    NestedMeta,
    MetaNameValue,
    DataStruct,
    Type,
    TypePath,
};
use std::fmt::{
    Debug,
    Formatter,
    Result as FmtResult,
};

use proc_macro2::Span;

#[cfg(feature = "diagnostic-notes")]
use proc_macro::{ Level, Diagnostic };

const ATTR_LITTLE_ENDIAN: &str = "little_endian";
const ATTR_BIG_ENDIAN: &str = "big_endian";
const ATTR_MSB0: &str = "msb0";
const ATTR_LSB0: &str = "lsb0";
const ATTR_BYTES: &str = "bytes";
const ATTR_WIDTH: &str = "width";
const ATTR_SPACE: &str = "space";
const ATTR_START_BYTE: &str = "start_byte";
const ATTR_END_BYTE: &str = "end_byte";
const ATTR_START_BIT: &str = "start_bit";
const ATTR_END_BIT: &str = "end_bit";


/// Derive for [Packed](../packing/trait.Packed.html)
#[proc_macro_derive(Packed, attributes(packed))]
pub fn packed_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    inner(input)
        .unwrap_or_else(|e| e.to_compile_error().into())
}

#[derive(Debug)]
enum Attr {
    Flag { name: Ident, span: Span },
    Value { name: Ident, value: Lit, span: Span },
    Lit { value: Lit, span: Span },
}

impl Attr {
    fn span(&self) -> Span {
        match self {
            Attr::Flag { span, .. } => span.clone(),
            Attr::Value { span, .. } => span.clone(),
            Attr::Lit { span, .. } => span.clone(),
        }
    }
}

fn get_single_segment(path: &Path) -> Result<Ident, Error> {
    if path.segments.len() != 1 {
        Err(Error::new(path.span(), "Expected 1 segments"))?
    }
    Ok(path.segments[0].ident.clone())
}

fn flatten_attrs(attrs: &Vec<Attribute>) -> Result<Vec<Attr>, Error> {
    let mut ret = Vec::new();

    for a in attrs.iter() {
        match a.parse_meta() {
            Ok(Meta::List(l)) => {
                for n in l.nested.iter() {
                    ret.push(match n {
                        NestedMeta::Meta(Meta::Path(p)) => 
                            Attr::Flag { name: get_single_segment(p)?, span: p.span() },
                        NestedMeta::Meta(Meta::NameValue(MetaNameValue { path, lit, .. })) => 
                            Attr::Value { name: get_single_segment(path)?, value: lit.clone(), span: path.span() },
                        NestedMeta::Lit(l) => 
                            Attr::Lit { value: l.clone(), span: a.span() },
                        y => panic!("y: {:?}", y),
                    });
                }
            },
            // This means #[packed] with no extra attributes
            Ok(Meta::Path(_)) => {},
            x => panic!("x: {:?}", x),
        }
    }

    Ok(ret)
}

trait Name {
    fn name() -> &'static str;
    fn instance_name(&self) -> &'static str;
}

trait TryFrom<T> {
    fn try_from(v: &T) -> Result<Self, Error> where Self: Sized;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Endian {
    Big,
    Little,
}
impl Default for Endian {
    fn default() -> Endian {
        Endian::Little
    }
}
impl Name for Endian {
    fn name() -> &'static str {
        "Endian"
    }
    fn instance_name(&self) -> &'static str {
        match self {
            Endian::Big => "big",
            Endian::Little => "little",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BitOrder {
    Msb0,
    Lsb0,
}
impl Default for BitOrder {
    fn default() -> BitOrder {
        BitOrder::Msb0
    }
}
impl Name for BitOrder {
    fn name() -> &'static str {
        "BitOrder"
    }
    fn instance_name(&self) -> &'static str {
        match self {
            BitOrder::Msb0 => "Msb0",
            BitOrder::Lsb0 => "Lsb0",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scope {
    Struct,
    Field,
}
impl Name for Scope {
    fn name() -> &'static str {
        "Scope"
    }
    fn instance_name(&self) -> &'static str {
        match self {
            Scope::Struct => "Struct",
            Scope::Field => "Field",
        }
    }
}

fn lit_to_usize(lit: &Lit) -> Result<usize, Error> {
    match lit {
        Lit::Int(i) => Ok(i.base10_parse()?),
        _ => Err(Error::new(lit.span(), format!("Expected usize literal but got: {:?}", lit))),
    }
}

impl TryFrom<Attr> for Option<(usize, Span)> {
    fn try_from(v: &Attr) -> Result<Option<(usize, Span)>, Error> {
        match v {
            Attr::Value { value, .. } => Ok(Some((lit_to_usize(value)?, value.span()))),
            _ => Err(Error::new(v.span(), format!("Expected Attr::Value but got: {:?}", v))),
        }
    }
}


macro_rules! usize_field {
    ($type: ident, $name: expr, $instance_name: expr) => {
        #[derive(Clone, Copy, Default)]
        struct $type (Option<(usize, Span)>);
        impl Name for $type {
            fn name() -> &'static str {
                $name
            }
            fn instance_name(&self) -> &'static str {
                $instance_name
            }
        }
        impl TryFrom<Attr> for $type {
            fn try_from(v: &Attr) -> Result<$type, Error> {
                Option::<(usize, Span)>
                    ::try_from(v)
                    .map(|u| Self(u))
            }
        }
        impl Debug for $type {
            fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
                match self.0 {
                    None => write!(f, "Unspecified"),
                    Some((b, ..)) => write!(f, "{} {}", b, self.instance_name()),
                }
            }
        }
        impl $type {
            #[allow(dead_code)]
            fn value(&self) -> Option<usize> {
                self.0.map(|x| x.0)
            }
        }
    }
}

usize_field!(Bytes, "Bytes", "Bytes");
usize_field!(Width, "Width", "Width");
usize_field!(Space, "Space", "Space");
usize_field!(StartByte, "StartByte", "StartByte");
usize_field!(EndByte, "EndByte", "EndByte");
usize_field!(StartBit, "StartBit", "StartBit");
usize_field!(EndBit, "EndBit", "EndBit");


fn get_attr<'a, I, Ta: 'a, Tb, F>(iter: I, _span: Span, scope: Scope, default: Tb, filter_map: F) -> Result<Tb, Error> 
where 
    I: Iterator<Item = &'a Ta>,
    Tb: Clone + Name + Debug,
    F: FnMut(&Ta) -> Option<(Result<Tb, Error>, Span)>,
{
    let results: Vec<(Tb, Span)> = iter.filter_map(filter_map).map(|r| match r {
        (Ok(r), span) => Ok((r, span)),
        (Err(e), span) => Err(Error::new(span, e)),
    }).collect::<Result<_, _>>()?;

    let scope = scope.instance_name();
    let name = Tb::name();

    //let multi_span: Vec<proc_macro::Span> = vec![span.unwrap()];
    match results.len() {
        0 => {
            let r = default;
            //Diagnostic::spanned(span.unwrap(), Level::Note, format!("{}.{} not specified, defaulting to {:?}", scope, name, r)).emit();
            Ok(r)
        },
        1 => {
            let (r, _span) = results[0].clone();
            //Diagnostic::spanned(span.unwrap(), Level::Note, format!("{}.{} specified as {:?}", scope, name, r)).emit();
            Ok(r)
        },
        _ => {
            #[cfg(feature = "diagnostic-notes")]
            {
                Diagnostic::spanned(results.iter().map(|x| x.1.unwrap()).collect::<Vec<proc_macro::Span>>(), 
                    Level::Error, format!("{}.{} specified multiple times", scope, name)).emit();
            }

            Err(Error::new(results[results.len()-1].1, format!("Multiple {}.{} is invalid", scope, name)))
        },
    }
}

fn get_value<'a, A, B>(attrs: A, span: Span, scope: Scope, name_: &str) -> Result<B, Error> 
where 
    A: Iterator<Item = &'a Attr>,
    B: TryFrom<Attr> + Debug + Clone + Default + Name
{
    get_attr(attrs, span, scope, Default::default(), |a| match a {
        Attr::Value { name, value, .. } if name == name_ => {
            Some((B::try_from(a), value.span()))
        },
        _ => None,
    })
}

fn get_endianness<'a, A>(attrs: A, span: Span, scope: Scope, default: Endian) -> Result<Endian, Error> 
where
    A: Iterator<Item = &'a Attr>
{
    get_attr(attrs, span, scope, default, |a| match a {
        Attr::Flag { name, span } if name == ATTR_LITTLE_ENDIAN  => Some((Ok(Endian::Little), span.clone())),
        Attr::Flag { name, span } if name == ATTR_BIG_ENDIAN => Some((Ok(Endian::Big), span.clone())),
        _ => None,
    })
}

fn get_bit_order<'a, A>(attrs: A, span: Span, scope: Scope) -> Result<BitOrder, Error> 
where
    A: Iterator<Item = &'a Attr>
{
    get_attr(attrs, span, scope, Default::default(), |a| match a {
        Attr::Flag { name, span } if name == ATTR_MSB0  => Some((Ok(BitOrder::Msb0), span.clone())),
        Attr::Flag { name, span } if name == ATTR_LSB0 => Some((Ok(BitOrder::Lsb0), span.clone())),
        _ => None,
    })
}

const SUPPORTED_FIELD_TYPES: [(&str, usize); 4] = [
    ("u8", 8),
    ("u16", 16),
    ("u32", 32),
    ("bool", 1),
];

fn get_bit_width(ident: &Ident) -> Result<usize, Error> {
    for (i, size) in SUPPORTED_FIELD_TYPES.iter() {
        if ident.eq(i) {
            return Ok(*size);
        }
    }
    Err(Error::new(ident.span(), format!("Unsupported field type {:?}", ident)))
}

struct Field {
    name: Ident,
    out_bits: usize,
    out_ident: Ident,
    out_type: TypePath,
    width: Width,
    space: Space,
    start_byte: StartByte,
    end_byte: EndByte,
    start_bit: StartBit,
    end_bit: EndBit,
    endian: Endian,
}

struct ExplicitField {
    name: Ident,
    out_type: TypePath,
    start_bit: usize,
    end_bit: usize,
    endian: Endian,
    width_bytes: usize,
    start_byte: usize,
    end_byte: usize,
}

impl Debug for Field {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, r"Field {{
    name: {},
    out_bits: {}, 
    width: {:?},
    space: {:?},
    start_byte: {:?},
    end_byte: {:?},
    start_bit: {:?},
    end_bit: {:?},
}}
", 
        self.name, self.out_bits, self.width.value(), self.space.value(), self.start_byte.value(),
        self.end_byte.value(), self.start_bit.value(), self.end_bit.value())
    }
}

fn error_or_diagnostic<M: core::fmt::Display>(span: Span, msg: M) -> Result<(), Error> {
    #[cfg(feature = "diagnostic-notes")]
    {
        Diagnostic::spanned(span.unwrap(), Level::Error, msg).emit();
        return Ok(());
    }

    #[cfg(not(feature = "diagnostic-notes"))]
    {
        return Err(Error::new(span, msg));
    }

}

fn inner(input: DeriveInput) -> Result<TokenStream, Error> {
    let struct_ident = input.ident.clone();
    let struct_span = input.ident.span();
    let DataStruct {
        //struct_token,
        fields,
        ..
    } = match input.data {
        Data::Struct(d) => { d },
        _ => Err(Error::new(struct_span, "Packed derive only supported on structs"))?,
    };

    let struct_attrs = flatten_attrs(&input.attrs)?;
    let struct_endian = get_endianness(struct_attrs.iter(), struct_span, Scope::Struct, Default::default())?;
    let bit_order = get_bit_order(struct_attrs.iter(), struct_span, Scope::Struct)?;
    let _bytes: Bytes = get_value(struct_attrs.iter(), struct_span, Scope::Struct, ATTR_BYTES)?;

    let map_bits = |(b, span): (usize, Span)| (match bit_order {
        BitOrder::Lsb0 => 7-b,
        BitOrder::Msb0 => b,
    }, span);

    let named_fields = if let Fields::Named(f) = fields { 
        f
    } else {
        // TODO: shouldn't be hard to support the other kinds
        Err(Error::new(struct_span, "Only named struct fields are accepted currently"))?
    };

    let mut fields = Vec::new();

    for f in named_fields.named {
        let attrs = flatten_attrs(&f.attrs)?;

        let ty = match f.ty {
            Type::Path(tp) => tp,
            _ => Err(Error::new(f.ident.span(), "Only Type::Path supported"))?,
        };

        let mut field = Field {
            name: f.ident.clone().unwrap(), // Since we checked it's a named struct above this is ok
            out_bits: get_bit_width(ty.path.get_ident().unwrap())?,
            out_ident: ty.path.get_ident().unwrap().clone(),
            out_type: ty,
            width: get_value(attrs.iter(), f.ident.span(), Scope::Field, ATTR_WIDTH)?,
            space: get_value(attrs.iter(), f.ident.span(), Scope::Field, ATTR_SPACE)?,
            start_byte: get_value(attrs.iter(), f.ident.span(), Scope::Field, ATTR_START_BYTE)?,
            end_byte: get_value(attrs.iter(), f.ident.span(), Scope::Field, ATTR_END_BYTE)?,
            start_bit: get_value(attrs.iter(), f.ident.span(), Scope::Field, ATTR_START_BIT)?,
            end_bit: get_value(attrs.iter(), f.ident.span(), Scope::Field, ATTR_END_BIT)?,  
            endian: get_endianness(attrs.iter(), f.ident.span(), Scope::Field, struct_endian)?,          
        };


        if let Some(eb) = field.end_bit.value() {
            if eb > 7 {
                Err(Error::new(field.end_bit.0.unwrap().1, 
                    "end_bit must be between 0 and 7 (inclusive)"))?; 
            } else {
                field.end_bit.0 = field.end_bit.0.map(map_bits);
            }
        }
        if let Some(sb) = field.start_bit.value() {
            if sb > 7 {
                Err(Error::new(field.start_bit.0.unwrap().1, 
                    "start_bit must be between 0 and 7 (inclusive)"))?; 
            } else {
                field.start_bit.0 = field.start_bit.0.map(map_bits);
            }
        }

        if let (Some(sb), Some(eb)) = (field.start_bit.value(), field.end_bit.value()) {
            if sb > eb {
                if bit_order == BitOrder::Lsb0 {
                    Err(Error::new(field.start_bit.0.unwrap().1,
                        "start_bit must be >= end_bit when using lsb0 bit order"))?;
                } else {
                    Err(Error::new(field.start_bit.0.unwrap().1,
                        "start_bit must be <= end_bit when using msb0 bit order"))?;
                }
            }
        }

        fields.push(field);

        //Diagnostic::spanned(f.ident.span().unwrap(), Level::Note, format!("Field: {:?}", field)).emit(); 
    }

    let mut pack_to_comment = "|byte|".to_string();
    match bit_order {
        BitOrder::Msb0 => for i in 0..=7 {
            pack_to_comment += &format!("{}|", i);
        },
        BitOrder::Lsb0 => for i in (0..=7_usize).rev() {
            pack_to_comment += &format!("{}|", i);
        },
    }
    pack_to_comment += "\n|-|-|-|-|-|-|-|-|-|\n";
    

    let mut explicit_fields = Vec::new();
    let mut bit = 0;

    let mut max_byte = 0;

    for f in fields {
        if let Some(b) = f.start_bit.value() {
            while bit % 8 != b {
                bit += 1;
            }
        }

        if let Some(b) = f.start_byte.value() {
            if b < bit / 8 {
                error_or_diagnostic(f.name.span(),
                    format!("Field start ({}) specified before current position ({}), are the fields out of order?",
                        b, bit/8))?;
            }
            while b > bit / 8 {
                bit += 8;
            }
        }

        let mut end = bit;
        let mut end_set = false;
        if let Some(b) = f.end_bit.value() {
            while end % 8 != b {
                end += 1;
            }
            end_set = true;
        }

        if let Some(b) = f.end_byte.value() {
            while b > end / 8 {
                end += 8;
            }
            end_set = true;
        }

        if let Some(w) = f.width.value() {
            if end_set {
                if w != end - bit {
                    error_or_diagnostic(f.name.span(),
                        format!("Field specifies width of {} but calculated width is {}. Check width, start/end byte/bit attributes", 
                            w, end - bit))?; 
                }
            } else {
                end += w;
                end_set = true;
            }
        }

        if !end_set {
            end += f.out_bits;

            #[cfg(feature = "diagnostic-notes")]
            Diagnostic::spanned(f.name.span().unwrap(), Level::Note, 
                format!("Field {} inferred length: {}", 
                    f.name, f.out_bits)).emit(); 

            panic!("!end_set: {:?}", f);
        }

        if end - bit > f.out_bits {
            error_or_diagnostic(f.name.span(),
                format!("Field width is {} bits which is more than will fit in {:?} ({} bits)", 
                    end - bit, f.out_ident.to_string(), f.out_bits))?; 
        }

        #[cfg(feature = "diagnostic-notes")]
        Diagnostic::spanned(f.name.span().unwrap(), Level::Note,
            format!("{}: {} -> {} ({}.{} .. {}.{})", f.name, bit, end, 
                f.start_byte.value().unwrap(),
                f.start_bit.value().unwrap(),
                f.end_byte.value().unwrap(),
                f.end_bit.value().unwrap(),
            )).emit();

        explicit_fields.push(ExplicitField {
            name: f.name,
            out_type: f.out_type,
            start_bit: bit,
            end_bit: end,
            endian: f.endian,
            width_bytes: (end - bit) / 8 + 1,
            start_byte: bit / 8,
            end_byte: end / 8,
        });

        bit = end;
        max_byte = max_byte.max(end / 8);
    }

    let (lsb, msb) = if bit_order == BitOrder::Lsb0 {
        (" LSB", " MSB")
    } else {
        (" MSB", " LSB")
    };

    bit = 0;
    for f in explicit_fields.iter() {
        for i in bit..=f.end_bit {
            pack_to_comment += "|";
            if i % 8 == 0 {
                pack_to_comment += &format!("{}|", i / 8);
            }
            if i == f.start_bit {
                pack_to_comment += &format!("{}", f.name);
                if f.start_bit != f.end_bit {
                    pack_to_comment += msb;
                }
            } else if i == f.end_bit {
                pack_to_comment += &format!("{}", f.name);
                if f.start_bit != f.end_bit {
                    pack_to_comment += lsb;
                }
            } else if i > f.start_bit && i < f.end_bit {
                pack_to_comment += " - ";
            }

            if i % 8 == 7 {
                pack_to_comment += "|\n";
            }
        }
        bit = f.end_bit + 1;
    }

    let min_len = max_byte + 1;

    pack_to_comment.insert_str(0, &format!("Pack into the provided byte slice.\n\n`bytes.len()` must be at least {}\n\n", min_len));

    let mut pack_comment = format!("Pack into a new byte array. Returns [u8; {}]\n\n", min_len);
    pack_comment += &format!("See [pack_to](struct.{}.html#method.pack_to) for layout diagram", struct_ident.to_string());

    let mut unpack_comment = format!("Unpack from byte slice into new instance.\n\n`bytes.len()` must be at least {}\n\n", min_len);
    unpack_comment += &format!("See [pack_to](struct.{}.html#method.pack_to) for layout diagram", struct_ident.to_string());

    let mut unpack_to_self = format!("Unpack from byte slice into self.\n\n`bytes.len()` must be at least {}\n\n", min_len);
    unpack_to_self += &format!("See [pack_to](struct.{}.html#method.pack_to) for layout diagram", struct_ident.to_string());

    let pack_bytes_len_comment = format!("Number of bytes this struct packs to/from ({})", min_len);

    let mut unpackers = Vec::new();

    let map_typenum = |b: usize| {
        match b {
            7 => quote!{U7},
            6 => quote!{U6},
            5 => quote!{U5},
            4 => quote!{U4},
            3 => quote!{U3},
            2 => quote!{U2},
            1 => quote!{U1},
            0 => quote!{U0},
            _ => unreachable!(),
        }
    };

    for f in explicit_fields.iter() {
        let name = &f.name;
        let ty = &f.out_type;
        let sbit = map_typenum(7-(f.start_bit % 8) );
        let ebit = map_typenum(7-(f.end_bit % 8) );
        let width = map_typenum(f.width_bytes);
        let sbyte = f.start_byte;
        let ebyte = f.end_byte;
        let endian = if f.endian == Endian::Little {
            format_ident!("LittleEndian")
        } else {
            format_ident!("BigEndian")
        };

        unpackers.push(quote! {            
            #name: <#ty as Packed<&[u8], #sbit, #ebit, #width>>::unpack::<packing::#endian>(&bytes[#sbyte..=#ebyte])?,   
        });
    }

    let width = map_typenum(min_len);
    let result = quote!{
        impl #struct_ident { 
            #[doc = #pack_to_comment]
            pub fn pack_to(&self, bytes: &mut [u8]) -> Result<(), packing::Error> {
                if bytes.len() < #min_len {
                    return Err(packing::Error::InsufficientBytes);
                }

                Ok(())
            }

            #[doc = #pack_comment]
            pub fn pack(&self) -> [u8; #min_len] {
                let mut bytes = [0; #min_len];
                self.pack_to(&mut bytes).unwrap();
                bytes
            }

            #[doc = #unpack_comment]
            pub fn unpack(bytes: &[u8]) -> Result<Self, packing::Error> {
                use packing::*;

                if bytes.len() < #min_len {
                    return Err(packing::Error::InsufficientBytes);
                }

                assert!(bytes.len() >= #min_len);

                Ok(Self {
                    #( #unpackers )*
                })
            }

            #[doc = #unpack_to_self]
            pub fn unpack_to_self(&mut self, bytes: &[u8]) -> Result<(), packing::Error> {
                *self = Self::unpack(bytes)?;
                Ok(())
            }
        }
        impl packing::Packed<&[u8], packing::U7, packing::U0, packing::#width> for #struct_ident {
            #[doc = #pack_bytes_len_comment]            
            const PACK_BYTES_LEN: usize = #min_len;
            fn unpack<E: packing::Endian>(bytes: &[u8]) -> Result<Self, packing::Error> {
                // TODO: allow S and E type parameters to vary
                // use case is for example a u8 with a few flags in it could be reused at
                // various offsets within a larger struct
                Self::unpack(bytes)
            }
        }
    };

    //panic!("{}", result.to_string());

    Ok(result.into())
}   