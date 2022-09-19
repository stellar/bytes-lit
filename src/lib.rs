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
/// The following integer literal forms are supported, preserve leading zeros in
/// the final byte representation and always return a consistent number of bytes
/// given the number of digits inputed.
/// - Base 2 (binary)
/// - Base 16 (hex)
///
/// For integer literal forms that preserve leading zeros, zeros on the front of
/// the number are preserved as zeros in the final bytes. For example: `0x0001`
/// will produce `[0, 1]`.
///
/// The following integer literal forms are supported, prohibit leading zeros,
/// and the number of bytes returned is not based off the number of digits
/// entered.
/// - Base 10 (decimal)
/// - Base 8 (octal)
///
/// For integer literal forms that do not have consistent digit to byte lengths,
/// the number of bytes returned is the minimum number of bytes required to
/// represent the integer.
///
/// ### Examples
///
/// ```ignore
/// let bytes = bytes!(1);
/// assert_eq!(bytes, [1]);
/// ```
///
/// ```ignore
/// let bytes = bytes!(9);
/// assert_eq!(bytes, [1]);
/// ```
///
/// ```ignore
/// let bytes = bytes!(0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d);
/// assert_eq!(bytes, [
///     253, 237, 63, 85, 222, 196, 114, 80, 165, 42, 140, 11, 183, 3, 142, 114,
///     250, 111, 250, 174, 51, 86, 47, 119, 205, 43, 98, 158, 247, 253, 66, 77,
/// ]);
/// ```
///
/// ```ignore
/// let bytes = bytes!(0x00000000dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d);
/// assert_eq!(bytes, [
///     0, 0, 0, 0, 222, 196, 114, 80, 165, 42, 140, 11, 183, 3, 142, 114,
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

    // Get the raw integer literal as it appears in the token stream.
    let raw = lit.to_string();

    // Remove leading negative sign, and any underscores between digits.
    let normalized = raw.replace(['-', '_'], "");

    // Remove any leading prefix that indicates the base, and use the base to
    // determine how many bits per leading zero needs to be prefilled into the
    // bytes generated.
    let (bits_per_digit, remainder) = match normalized.as_bytes() {
        [b'0', b'x', r @ ..] => (Some(4), r),
        [b'0', b'b', r @ ..] => (Some(1), r),
        [b'0', b'o', r @ ..] | [r @ ..] => (None, r),
    };

    // Count the leading zero bits by counting the number of leading zeros and
    // multiplying by the bits per digit.
    let leading_zero_count = remainder.iter().take_while(|d| **d == b'0').count();
    let leading_zero_bits = leading_zero_count * bits_per_digit.unwrap_or(0);

    // Convert the integer literal into a base10 number in a string. Any leading
    // zeros are discarded.
    let base10 = lit.base10_digits();

    // Convert the string base10 numbers into a big integer. The conversion
    // should never fail because syn::LitInt already validates the integer.
    let int = BigUint::from_str(base10).unwrap();
    let int_bits: usize = int.bits().try_into().expect("overflow");
    let int_bytes = int.to_bytes_be();
    let int_len = int_bytes.len();

    let total_bits = leading_zero_bits.checked_add(int_bits).expect("overflow");
    let total_len = (total_bits.checked_add(7).expect("overflow")) / 8;
    let mut total_bytes: Vec<u8> = vec![0; total_len];
    total_bytes[total_len - int_len..].copy_from_slice(&int_bytes);

    quote! { [#(#total_bytes),*] }
}

#[cfg(test)]
mod test {
    use crate::bytes2;
    use pretty_assertions::assert_eq;
    use quote::quote;
    use syn::{parse_quote, ExprArray};

    #[test]
    fn leading_zeros() {
        let table: &[(_, ExprArray)] = &[
            // Base 16.
            (quote!(0x1), parse_quote!([1u8])),
            (quote!(0x01), parse_quote!([1u8])),
            (quote!(0x0001), parse_quote!([0u8, 1u8])),
            (quote!(0x0_0_0_1), parse_quote!([0u8, 1u8])),
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
            (quote!(0b000000001), parse_quote!([0u8, 1u8])),
            (quote!(0o377), parse_quote!([255u8])),
            (quote!(0o0377), parse_quote!([255u8])),
            (quote!(0o00377), parse_quote!([255u8])),
            (quote!(0o400), parse_quote!([1u8, 0u8])),
        ];
        for (i, t) in table.iter().enumerate() {
            let tokens = bytes2(t.0.clone());
            let parsed = syn::parse2::<ExprArray>(tokens).unwrap();
            let expect = t.1.clone();
            assert_eq!(parsed, expect, "table entry: {}", i);
        }
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
