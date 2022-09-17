//! Bytes converts literals into an array of bytes.
//!
//! Currently supports only integer literals of unbounded size.

use std::{convert::TryInto, str::FromStr};

use num_bigint::BigUint;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::LitInt;

extern crate proc_macro;

/// Bytes converts literals into an array of bytes.
///
/// Currently supports only integer literals of unbounded size.
///
/// ### Examples
///
/// ```ignore
/// let bytes = bytes!(0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d);
/// assert_eq!(bytes, [
///     253, 237, 63, 85, 222, 196, 114, 80, 165, 42, 140, 11, 183, 3, 142, 114,
///     250, 111, 250, 174, 51, 86, 47, 119, 205, 43, 98, 158, 247, 253, 66, 77,
/// ]);
/// ```
#[proc_macro]
pub fn bytes(input: TokenStream) -> TokenStream {
    bytes2(input.into()).into()
}

fn bytes2(input: TokenStream2) -> TokenStream2 {
    let lit = match syn::parse2::<LitInt>(input) {
        Ok(lit) => lit,
        Err(e) => return e.to_compile_error(),
    };

    let raw = lit.to_string();
    let normalized = raw.replace(['-', '_'], "");
    let (bits_per_digit, prefix_len) = if normalized.starts_with("0x") {
        (4 /* base 16 */, 2)
    } else if raw.starts_with("0o") {
        (2 /* base 8 */, 2)
    } else if raw.starts_with("0b") {
        (1 /* base 2 */, 2)
    } else {
        (4 /* base 10 */, 0)
    };
    let mut leading_zero_count: u64 = 0;
    for d in normalized[prefix_len..].chars() {
        if d != '0' {
            break;
        }
        leading_zero_count += 1;
    }
    let leading_zero_bits = bits_per_digit * leading_zero_count;

    let int = BigUint::from_str(lit.base10_digits()).unwrap();
    let int_bits = int.bits();
    let int_bytes = int.to_bytes_be();
    let int_len = int_bytes.len();

    let total_bits = leading_zero_bits.checked_add(int_bits).expect("overflow");
    let total_len_u64 = (total_bits.checked_add(7).expect("overflow")) / 8;
    let total_len: usize = total_len_u64.try_into().expect("overflow");
    let mut total_bytes: Vec<u8> = vec![0; total_len];
    total_bytes[total_len - int_len..].copy_from_slice(&int_bytes);

    quote! { [#(#total_bytes),*] }
}

#[cfg(test)]
mod test {
    use crate::bytes2;
    use pretty_assertions::assert_eq;
    use quote::quote;
    use syn::ExprArray;

    #[test]
    fn leading_zeros() {
        let tokens = bytes2(quote! {0x00_928374892abc});
        let parsed = syn::parse2::<ExprArray>(tokens).unwrap();
        let expect = syn::parse_quote!([0u8, 146u8, 131u8, 116u8, 137u8, 42u8, 188u8]);
        assert_eq!(parsed, expect);

        let tokens = bytes2(quote! {0x00_92837_4892abcE100});
        let parsed = syn::parse2::<ExprArray>(tokens).unwrap();
        let expect = syn::parse_quote!([0u8, 146u8, 131u8, 116u8, 137u8, 42u8, 188u8, 225u8, 0u8]);
        assert_eq!(parsed, expect);

        let tokens = bytes2(quote! {0b1});
        let parsed = syn::parse2::<ExprArray>(tokens).unwrap();
        let expect = syn::parse_quote!([1u8]);
        assert_eq!(parsed, expect);
    }

    #[test]
    fn hex() {
        let tokens = bytes2(quote! {0x1});
        let parsed = syn::parse2::<ExprArray>(tokens).unwrap();
        let expect = syn::parse_quote!([1u8]);
        assert_eq!(parsed, expect);

        let tokens = bytes2(quote! {0x928374892abc});
        let parsed = syn::parse2::<ExprArray>(tokens).unwrap();
        let expect = syn::parse_quote!([146u8, 131u8, 116u8, 137u8, 42u8, 188u8]);
        assert_eq!(parsed, expect);

        let tokens =
            bytes2(quote! {0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d});
        let parsed = syn::parse2::<ExprArray>(tokens).unwrap();
        let expect = syn::parse_quote!([
            253u8, 237u8, 63u8, 85u8, 222u8, 196u8, 114u8, 80u8, 165u8, 42u8, 140u8, 11u8, 183u8,
            3u8, 142u8, 114u8, 250u8, 111u8, 250u8, 174u8, 51u8, 86u8, 47u8, 119u8, 205u8, 43u8,
            98u8, 158u8, 247u8, 253u8, 66u8, 77u8
        ]);
        assert_eq!(parsed, expect);
    }

    #[test]
    fn base10() {
        let tokens = bytes2(quote! {340_282_366_920_938_463_463_374_607_431_768_211_455u128});
        let parsed = syn::parse2::<ExprArray>(tokens).unwrap();
        let expect = syn::parse_quote!([
            255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
            255u8, 255u8, 255u8, 255u8
        ]);
        assert_eq!(parsed, expect);

        let tokens = bytes2(quote! {340_282_366_920_938_463_463_374_607_431_768_211_456});
        let parsed = syn::parse2::<ExprArray>(tokens).unwrap();
        let expect = syn::parse_quote!([
            1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8
        ]);
        assert_eq!(parsed, expect);
    }
}
