//! Bytes converts literals into an array of bytes.
//!
//! Currently supports only integer literals of unbounded size.

use std::str::FromStr;

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
    let int = BigUint::from_str(lit.base10_digits()).unwrap();
    let bytes = int.to_bytes_be();
    quote! { [#(#bytes),*] }
}

#[cfg(test)]
mod test {
    use crate::bytes2;
    use pretty_assertions::assert_eq;
    use quote::quote;
    use syn::ExprArray;

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
