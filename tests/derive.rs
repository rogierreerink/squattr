#[cfg(test)]
mod tests {
    use squattr::{attribute::AttributeExt, derive::Squattr};

    use proc_macro2::Span;
    use quote::quote;
    use syn::{Ident, Lit, LitBool, LitFloat, LitInt, LitStr};

    #[test]
    fn parse_attributes_derived() {
        #[derive(Squattr, PartialEq, Debug)]
        pub struct TestAttribute {
            tst_usize: usize,
            tst_u128: u128,
            tst_u64: u64,
            tst_u32: u32,
            tst_u16: u16,
            tst_u8: u8,
            tst_isize: isize,
            tst_i128: i128,
            tst_i64: i64,
            tst_i32: i32,
            tst_i16: i16,
            tst_i8: i8,
            tst_f64: f64,
            tst_f32: f32,

            tst_usize_list: Vec<usize>,
            tst_u128_list: Vec<u128>,
            tst_u64_list: Vec<u64>,
            tst_u32_list: Vec<u32>,
            tst_u16_list: Vec<u16>,
            tst_u8_list: Vec<u8>,
            tst_isize_list: Vec<isize>,
            tst_i128_list: Vec<i128>,
            tst_i64_list: Vec<i64>,
            tst_i32_list: Vec<i32>,
            tst_i16_list: Vec<i16>,
            tst_i8_list: Vec<i8>,
            tst_f64_list: Vec<f64>,
            tst_f32_list: Vec<f32>,

            tst_bool: bool,
            tst_str: String,
            tst_str_list: Vec<String>,

            // Make these optional, as they need to implement `Default`:
            tst_ident: Option<Ident>,
            tst_lit: Option<Lit>,
            tst_lit_bool: Option<LitBool>,
            tst_lit_float: Option<LitFloat>,
            tst_lit_int: Option<LitInt>,
            tst_lit_str: Option<LitStr>,
        }

        #[derive(Squattr, PartialEq, Debug)]
        pub struct SubAttribute {
            some_sub_bool: Option<bool>,
        }

        let usize_max = usize::MAX;
        let u128_max = u128::MAX;
        let u64_max = u64::MAX;
        let u32_max = u32::MAX;
        let u16_max = u16::MAX;
        let u8_max = u8::MAX;
        let usize_min = usize::MIN;
        let u128_min = u128::MIN;
        let u64_min = u64::MIN;
        let u32_min = u32::MIN;
        let u16_min = u16::MIN;
        let u8_min = u8::MIN;

        let isize_max = isize::MAX;
        let i128_max = i128::MAX;
        let i64_max = i64::MAX;
        let i32_max = i32::MAX;
        let i16_max = i16::MAX;
        let i8_max = i8::MAX;
        let f64_max = f64::MAX;
        let f32_max = f32::MAX;
        let isize_min = isize::MIN;
        let i128_min = i128::MIN;
        let i64_min = i64::MIN;
        let i32_min = i32::MIN;
        let i16_min = i16::MIN;
        let i8_min = i8::MIN;
        let f64_min = f64::MIN;
        let f32_min = f32::MIN;

        let input = quote! {
            tst_usize = #usize_max,
            tst_u128 = #u128_max,
            tst_u64 = #u64_max,
            tst_u32 = #u32_max,
            tst_u16 = #u16_max,
            tst_u8 = #u8_max,
            tst_isize = #isize_min,
            tst_i128 = #i128_min,
            tst_i64 = #i64_min,
            tst_i32 = #i32_min,
            tst_i16 = #i16_min,
            tst_i8 = #i8_min,
            tst_f64 = #f64_min,
            tst_f32 = #f32_min,

            tst_usize_list(#usize_min, #usize_max),
            tst_u128_list(#u128_min, #u128_max),
            tst_u64_list(#u64_min, #u64_max),
            tst_u32_list(#u32_min, #u32_max),
            tst_u16_list(#u16_min, #u16_max),
            tst_u8_list(#u8_min, #u8_max),
            tst_isize_list(#isize_min, #isize_max),
            tst_i128_list(#i128_min, #i128_max),
            tst_i64_list(#i64_min, #i64_max),
            tst_i32_list(#i32_min, #i32_max),
            tst_i16_list(#i16_min, #i16_max),
            tst_i8_list(#i8_min, #i8_max),
            tst_f64_list(#f64_min, #f64_max),
            tst_f32_list(#f32_min, #f32_max),

            tst_bool,
            tst_str = "foo",
            tst_str_list("foo", "bar"),

            tst_ident,
            tst_lit = "literal",
            tst_lit_bool = true,
            tst_lit_float = 123.456,
            tst_lit_int = 123,
            tst_lit_str = "literal",
        };

        pretty_assertions::assert_eq!(
            input.parse_attribute::<TestAttribute>().unwrap(),
            TestAttribute {
                tst_usize: usize::MAX,
                tst_u128: u128::MAX,
                tst_u64: u64::MAX,
                tst_u32: u32::MAX,
                tst_u16: u16::MAX,
                tst_u8: u8::MAX,
                tst_isize: isize::MIN,
                tst_i128: i128::MIN,
                tst_i64: i64::MIN,
                tst_i32: i32::MIN,
                tst_i16: i16::MIN,
                tst_i8: i8::MIN,
                tst_f64: f64::MIN,
                tst_f32: f32::MIN,

                tst_usize_list: vec![usize::MIN, usize::MAX],
                tst_u128_list: vec![u128::MIN, u128::MAX],
                tst_u64_list: vec![u64::MIN, u64::MAX],
                tst_u32_list: vec![u32::MIN, u32::MAX],
                tst_u16_list: vec![u16::MIN, u16::MAX],
                tst_u8_list: vec![u8::MIN, u8::MAX],
                tst_isize_list: vec![isize::MIN, isize::MAX],
                tst_i128_list: vec![i128::MIN, i128::MAX],
                tst_i64_list: vec![i64::MIN, i64::MAX],
                tst_i32_list: vec![i32::MIN, i32::MAX],
                tst_i16_list: vec![i16::MIN, i16::MAX],
                tst_i8_list: vec![i8::MIN, i8::MAX],
                tst_f64_list: vec![f64::MIN, f64::MAX],
                tst_f32_list: vec![f32::MIN, f32::MAX],

                tst_bool: true,
                tst_str: "foo".into(),
                tst_str_list: vec!["foo".into(), "bar".into()],

                tst_ident: Some(Ident::new("tst_ident", Span::call_site())),
                tst_lit: Some(Lit::Str(LitStr::new("literal", Span::call_site()))),
                tst_lit_bool: Some(LitBool::new(true, Span::call_site())),
                tst_lit_float: Some(LitFloat::new("123.456", Span::call_site())),
                tst_lit_int: Some(LitInt::new("123", Span::call_site())),
                tst_lit_str: Some(LitStr::new("literal", Span::call_site())),
            }
        );
    }
}
