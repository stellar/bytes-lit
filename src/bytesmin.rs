use std::str::FromStr;

use num_bigint::BigUint;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Error, LitInt};

pub fn bytesmin(input: TokenStream2) -> TokenStream2 {
    let lit = match syn::parse2::<LitInt>(input) {
        Ok(lit) => lit,
        Err(e) => return e.to_compile_error(),
    };

    // Reject unsupported literal forms.
    let raw = lit.to_string();
    let normalized = raw.replace('_', "");
    match normalized.as_bytes() {
        [b'0', b'x', ..] | [b'0', b'b', ..] => {}
        _ => {
            return Error::new(
                lit.span(),
                "only positive hex (0x) and binary (0b) integer literals are supported",
            )
            .to_compile_error();
        }
    }

    // The conversion should never fail because syn::LitInt already validated
    // the integer, and the form check above ensures only non-negative hex/binary
    // literals reach here.
    let int = BigUint::from_str(lit.base10_digits()).expect("valid hex or binary literal");
    let bytes = int.to_bytes_be();
    quote! { [#(#bytes),*] }
}

#[cfg(test)]
mod test {
    use super::bytesmin;
    use pretty_assertions::assert_eq;
    use proc_macro2::Span;
    use quote::quote;
    use syn::{parse_quote, Error, ExprArray};

    #[test]
    fn neg() {
        let tokens = bytesmin(quote! {-0x1});
        let expect = Error::new(
            Span::call_site(),
            "only positive hex (0x) and binary (0b) integer literals are supported",
        )
        .to_compile_error()
        .to_string();
        assert_eq!(tokens.to_string(), expect);
    }

    #[test]
    fn hex() {
        let tokens = bytesmin(quote! {0x1});
        let parsed = syn::parse2::<ExprArray>(tokens).unwrap();
        let expect = syn::parse_quote!([1u8]);
        assert_eq!(parsed, expect);

        let tokens = bytesmin(quote! {0x928374892abc});
        let parsed = syn::parse2::<ExprArray>(tokens).unwrap();
        let expect = syn::parse_quote!([146u8, 131u8, 116u8, 137u8, 42u8, 188u8]);
        assert_eq!(parsed, expect);

        let tokens =
            bytesmin(quote! {0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d});
        let parsed = syn::parse2::<ExprArray>(tokens).unwrap();
        let expect = syn::parse_quote!([
            253u8, 237u8, 63u8, 85u8, 222u8, 196u8, 114u8, 80u8, 165u8, 42u8, 140u8, 11u8, 183u8,
            3u8, 142u8, 114u8, 250u8, 111u8, 250u8, 174u8, 51u8, 86u8, 47u8, 119u8, 205u8, 43u8,
            98u8, 158u8, 247u8, 253u8, 66u8, 77u8
        ]);
        assert_eq!(parsed, expect);
    }

    #[test]
    fn decimal_and_octal_unsupported() {
        let table: &[_] = &[
            // Decimal.
            quote!(0),
            quote!(1),
            quote!(9),
            quote!(255),
            quote!(256),
            quote!(0255),
            quote!(00255),
            quote!(00),
            quote!(340_282_366_920_938_463_463_374_607_431_768_211_455u128),
            quote!(340_282_366_920_938_463_463_374_607_431_768_211_456),
            // Octal.
            quote!(0o0),
            quote!(0o1),
            quote!(0o377),
            quote!(0o0377),
            quote!(0o00377),
            quote!(0o00),
            quote!(0o400),
        ];
        let expect = Error::new(
            Span::call_site(),
            "only positive hex (0x) and binary (0b) integer literals are supported",
        )
        .to_compile_error()
        .to_string();
        for (i, input) in table.iter().enumerate() {
            let tokens = bytesmin(input.clone());
            assert_eq!(tokens.to_string(), expect, "table entry: {}", i);
        }
    }

    #[test]
    fn zero() {
        let table: &[(_, ExprArray)] = &[
            (quote!(0x0), parse_quote!([0u8])),
            (quote!(0x00), parse_quote!([0u8])),
            (quote!(0b0), parse_quote!([0u8])),
            (quote!(0b00000000), parse_quote!([0u8])),
        ];
        for (i, t) in table.iter().cloned().enumerate() {
            let tokens = bytesmin(t.0);
            let parsed = syn::parse2::<ExprArray>(tokens).unwrap();
            let expect = t.1;
            assert_eq!(parsed, expect, "table entry: {}", i);
        }
    }

    #[test]
    fn byte_boundaries() {
        let table: &[(_, ExprArray)] = &[
            // u8 max
            (quote!(0xff), parse_quote!([255u8])),
            (quote!(0b11111111), parse_quote!([255u8])),
            // u8 max + 1
            (quote!(0x100), parse_quote!([1u8, 0u8])),
            (quote!(0b100000000), parse_quote!([1u8, 0u8])),
            // u16 max
            (quote!(0xffff), parse_quote!([255u8, 255u8])),
            // u16 max + 1
            (quote!(0x10000), parse_quote!([1u8, 0u8, 0u8])),
            // u32 max
            (
                quote!(0xffffffff),
                parse_quote!([255u8, 255u8, 255u8, 255u8]),
            ),
            // u32 max + 1
            (quote!(0x100000000), parse_quote!([1u8, 0u8, 0u8, 0u8, 0u8])),
            // u64 max
            (
                quote!(0xffffffffffffffff),
                parse_quote!([255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8]),
            ),
            // u64 max + 1
            (
                quote!(0x10000000000000000),
                parse_quote!([1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8]),
            ),
            // u128 max
            (
                quote!(0xffffffffffffffffffffffffffffffff),
                parse_quote!([
                    255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
                    255u8, 255u8, 255u8, 255u8, 255u8
                ]),
            ),
            // u128 max + 1
            (
                quote!(0x100000000000000000000000000000000),
                parse_quote!([
                    1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
                    0u8
                ]),
            ),
        ];
        for (i, t) in table.iter().cloned().enumerate() {
            let tokens = bytesmin(t.0);
            let parsed = syn::parse2::<ExprArray>(tokens).unwrap();
            let expect = t.1;
            assert_eq!(parsed, expect, "table entry: {}", i);
        }
    }

    #[test]
    fn one() {
        let table: &[(_, ExprArray)] = &[
            (quote!(0x1), parse_quote!([1u8])),
            (quote!(0b1), parse_quote!([1u8])),
        ];
        for (i, t) in table.iter().cloned().enumerate() {
            let tokens = bytesmin(t.0);
            let parsed = syn::parse2::<ExprArray>(tokens).unwrap();
            let expect = t.1;
            assert_eq!(parsed, expect, "table entry: {}", i);
        }
    }

    #[test]
    fn empty_input() {
        let tokens = bytesmin(quote! {});
        let expect = Error::new(
            Span::call_site(),
            "unexpected end of input, expected integer literal",
        )
        .to_compile_error()
        .to_string();
        assert_eq!(tokens.to_string(), expect);
    }

    #[test]
    fn non_integer_input() {
        let tokens = bytesmin(quote! {"hello"});
        let expect = Error::new(Span::call_site(), "expected integer literal")
            .to_compile_error()
            .to_string();
        assert_eq!(tokens.to_string(), expect);
    }

    #[test]
    fn leading_zeros_discarded() {
        let table: &[(_, ExprArray)] = &[
            // Base 16.
            (quote!(0x1), parse_quote!([1u8])),
            (quote!(0x01), parse_quote!([1u8])),
            (quote!(0x0001), parse_quote!([1u8])),
            (quote!(0x0_0_0_1), parse_quote!([1u8])),
            (quote!(0x1u32), parse_quote!([1u8])),
            (quote!(0x01u32), parse_quote!([1u8])),
            (quote!(0x0001u32), parse_quote!([1u8])),
            (quote!(0x0_0_0_1u32), parse_quote!([1u8])),
            // Base 2.
            (quote!(0b1), parse_quote!([1u8])),
            (quote!(0b11), parse_quote!([3u8])),
            (quote!(0b111), parse_quote!([7u8])),
            (quote!(0b1111), parse_quote!([15u8])),
            (quote!(0b11111), parse_quote!([31u8])),
            (quote!(0b111111), parse_quote!([63u8])),
            (quote!(0b1111111), parse_quote!([127u8])),
            (quote!(0b11111111), parse_quote!([255u8])),
            (quote!(0b111111111), parse_quote!([1u8, 255u8])),
            (quote!(0b1), parse_quote!([1u8])),
            (quote!(0b01), parse_quote!([1u8])),
            (quote!(0b001), parse_quote!([1u8])),
            (quote!(0b0001), parse_quote!([1u8])),
            (quote!(0b00001), parse_quote!([1u8])),
            (quote!(0b000001), parse_quote!([1u8])),
            (quote!(0b0000001), parse_quote!([1u8])),
            (quote!(0b00000001), parse_quote!([1u8])),
            (quote!(0b000000001), parse_quote!([1u8])),
        ];
        for (i, t) in table.iter().cloned().enumerate() {
            let tokens = bytesmin(t.0);
            let parsed = syn::parse2::<ExprArray>(tokens).unwrap();
            let expect = t.1;
            assert_eq!(parsed, expect, "table entry: {}", i);
        }
    }
}
