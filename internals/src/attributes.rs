use proc_macro2::{Span, TokenStream};
use syn::{Error, parse::ParseStream};

use crate::ast::Values;

pub trait ParseAttributeExt: Sized {
    fn parse_attribute<T: TryFrom<(Values, Span), Error = Error>>(self) -> syn::Result<T>;
}

impl ParseAttributeExt for TokenStream {
    fn parse_attribute<T: TryFrom<(Values, Span), Error = Error>>(self) -> syn::Result<T> {
        syn::parse::Parser::parse2(
            |input: ParseStream| T::try_from((input.parse()?, input.span())),
            self,
        )
    }
}

#[cfg(test)]
mod tests {
    use proc_macro2::Span;
    use quote::quote;
    use syn::{Error, Ident, Lit, LitInt, Result};

    use crate::{
        ast::{List, Value, Values},
        attributes::ParseAttributeExt,
        errors::CombineErrorsExt,
        types::{FromValueExt, format_error},
    };

    #[derive(PartialEq, Debug)]
    pub struct SomeAttribute {
        pub some_list: Vec<String>,
        pub some_ident_list: Vec<Ident>,
        pub some_bool: bool,
        pub some_expr: Option<String>,
        pub some_ident: Option<Ident>,
        pub some_lit: Option<Lit>,
        pub some_sub_attr: Option<SubAttribute>,
    }

    impl TryFrom<(Values, Span)> for SomeAttribute {
        type Error = Error;

        fn try_from((values, span): (Values, Span)) -> Result<Self> {
            let mut errors = Vec::new();

            let mut some_list: Option<Vec<String>> = None;
            let mut some_ident_list: Option<Vec<Ident>> = None;
            let mut some_bool: Option<bool> = None;
            let mut some_expr: Option<String> = None;
            let mut some_ident: Option<Ident> = None;
            let mut some_lit: Option<Lit> = None;
            let mut some_sub_attr: Option<SubAttribute> = None;

            for value in values {
                let id = match value.identifier() {
                    Some(id) => id,
                    None => continue,
                };

                match id.as_str() {
                    id_str if id_str == "some_list" => {
                        some_list.from_value(id_str, value, &mut errors);
                    }
                    id_str if id_str == "some_ident_list" => {
                        some_ident_list.from_value(id_str, value, &mut errors);
                    }
                    id_str if id_str == "some_bool" => {
                        some_bool.from_value(id_str, value, &mut errors);
                    }
                    id_str if id_str == "some_expr" => {
                        some_expr.from_value(id_str, value, &mut errors);
                    }
                    id_str if id_str == "some_ident" => {
                        some_ident.from_value(id_str, value, &mut errors);
                    }
                    id_str if id_str == "some_lit" => {
                        some_lit.from_value(id_str, value, &mut errors);
                    }
                    id_str if id_str == "some_sub_attr" => {
                        some_sub_attr.from_value(id_str, value, &mut errors);
                    }
                    id_str => {
                        errors.push(Error::new(
                            value.span(),
                            format!("unrecognized entry `{}`", id_str),
                        ));
                    }
                }
            }

            if some_bool.is_none() {
                errors.push(Error::new(span, "expected key `some_expr` not found"));
            };
            if some_expr.is_none() {
                errors.push(Error::new(span, "expected key `some_expr` not found"));
            };

            if let Some(error) = errors.combine_errors() {
                return Err(error);
            }

            Ok(Self {
                some_list: some_list.unwrap_or_default(),
                some_ident_list: some_ident_list.unwrap_or_default(),
                some_bool: some_bool.unwrap_or_default(),
                some_expr,
                some_ident,
                some_lit,
                some_sub_attr,
            })
        }
    }

    impl TryFrom<Value> for SomeAttribute {
        type Error = Error;

        fn try_from(value: Value) -> Result<Self> {
            let span = value.span();

            let values = match value {
                Value::List(List { values, .. }) => values,
                value => return Err(format_error(&value, "a list of values")),
            };

            Self::try_from((values, span))
        }
    }

    #[derive(PartialEq, Debug)]
    pub struct SubAttribute {
        pub some_sub_bool: bool,
    }

    impl TryFrom<(Values, Span)> for SubAttribute {
        type Error = Error;

        fn try_from((values, span): (Values, Span)) -> Result<Self> {
            let mut errors = Vec::new();

            let mut some_sub_bool: Option<bool> = None;

            for value in values {
                let id = match value.identifier() {
                    Some(id) => id,
                    None => continue,
                };

                match id.as_str() {
                    id_str if id_str == "some_sub_bool" => {
                        some_sub_bool.from_value(id_str, value, &mut errors);
                    }
                    id_str => {
                        errors.push(Error::new(
                            value.span(),
                            format!("unrecognized entry `{}`", id_str),
                        ));
                    }
                }
            }

            if some_sub_bool.is_none() {
                errors.push(Error::new(span, "expected key `some_expr` not found"));
            };

            if let Some(error) = errors.combine_errors() {
                return Err(error);
            }

            Ok(Self {
                some_sub_bool: some_sub_bool.unwrap_or_default(),
            })
        }
    }

    impl TryFrom<Value> for SubAttribute {
        type Error = Error;

        fn try_from(value: Value) -> Result<Self> {
            let span = value.span();

            let values = match value {
                Value::List(List { values, .. }) => values,
                value => return Err(format_error(&value, "a list of values")),
            };

            Self::try_from((values, span))
        }
    }

    #[test]
    fn parse_attribute() {
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
                    some_sub_bool: false
                }),
            }
        );
    }
}
