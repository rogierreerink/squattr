#[cfg(test)]
mod tests {
    use squattr::{attribute::AttributeExt, derive::Squattr};

    use proc_macro2::Span;
    use quote::quote;
    use syn::{Ident, Lit, LitInt};

    #[test]
    fn parse_attributes_derived() {
        #[derive(Squattr, PartialEq, Debug)]
        pub struct SomeAttribute {
            pub some_list: Vec<String>,
            pub some_ident_list: Vec<Ident>,
            pub some_bool: bool,
            pub some_expr: Option<String>,
            pub some_ident: Option<Ident>,
            pub some_lit: Option<Lit>,
            pub some_sub_attr: Option<SubAttribute>,
            pub some_usize: usize,
            pub some_isize: isize,
            pub some_f64: f64,
            pub some_f64_as_u64: Option<u64>,
            pub some_u64_as_f64: f64,
        }

        #[derive(Squattr, PartialEq, Debug)]
        pub struct SubAttribute {
            pub some_sub_bool: Option<bool>,
        }

        let input = quote! {
            some_list("lit1", "lit2"),
            some_ident_list(id1, id2),
            some_bool,
            some_expr = "foo",
            some_ident,
            some_lit = 123,
            some_sub_attr(
                some_sub_bool = false
            ),
            some_usize = 1234,
            some_isize = -1234,
            some_f64 = 12.34,
            some_f64_as_u64 = 12.34,
            some_u64_as_f64 = 1234,
        };

        assert_eq!(
            input.parse_attribute::<SomeAttribute>().unwrap(),
            SomeAttribute {
                some_list: vec!["lit1".into(), "lit2".into()],
                some_ident_list: vec![
                    Ident::new("id1", Span::call_site()),
                    Ident::new("id2", Span::call_site())
                ],
                some_bool: true,
                some_expr: Some("foo".into()),
                some_ident: Some(Ident::new("some_ident", Span::call_site())),
                some_lit: Some(Lit::Int(LitInt::new("123", Span::call_site()))),
                some_sub_attr: Some(SubAttribute {
                    some_sub_bool: Some(false)
                }),
                some_usize: 1234,
                some_isize: -1234,
                some_f64: 12.34,
                some_f64_as_u64: Some(0),
                some_u64_as_f64: 1234.,
            }
        );
    }
}
