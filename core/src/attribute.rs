use proc_macro2::{Span, TokenStream};
use syn::{Result, parse::ParseStream};

use crate::{
    ast::{List, Value, Values},
    types::{ParseValue, format_error},
};

pub trait Attribute: Sized {
    fn parse(values: Values, span: Span) -> Result<Self>;
}

impl<T> ParseValue for T
where
    T: Attribute,
{
    fn parse(value: Value) -> Result<Self> {
        let span = value.span();
        let values = match value {
            Value::List(List { values, .. }) => values,
            value => return Err(format_error(&value, "a list of values")),
        };
        T::parse(values, span)
    }
}

pub trait AttributeStreamExt: Sized {
    fn parse_attribute<T: Attribute>(self) -> Result<T>;
}

impl AttributeStreamExt for TokenStream {
    fn parse_attribute<T: Attribute>(self) -> Result<T> {
        syn::parse::Parser::parse2(
            |input: ParseStream| T::parse(input.parse()?, input.span()),
            self,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::Values,
        attribute::{Attribute, AttributeStreamExt},
        errors::CombineErrorsExt,
        types::ValueStorageExt,
    };

    use proc_macro2::Span;
    use quote::quote;
    use syn::{Error, Ident, Lit, LitInt, Result};

    #[test]
    fn parse_attributes() {
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

        impl Attribute for SomeAttribute {
            fn parse(values: Values, span: Span) -> syn::Result<Self> {
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
                            some_list.insert_value(id_str, value, &mut errors);
                        }
                        id_str if id_str == "some_ident_list" => {
                            some_ident_list.insert_value(id_str, value, &mut errors);
                        }
                        id_str if id_str == "some_bool" => {
                            some_bool.insert_value(id_str, value, &mut errors);
                        }
                        id_str if id_str == "some_expr" => {
                            some_expr.insert_value(id_str, value, &mut errors);
                        }
                        id_str if id_str == "some_ident" => {
                            some_ident.insert_value(id_str, value, &mut errors);
                        }
                        id_str if id_str == "some_lit" => {
                            some_lit.insert_value(id_str, value, &mut errors);
                        }
                        id_str if id_str == "some_sub_attr" => {
                            some_sub_attr.insert_value(id_str, value, &mut errors);
                        }
                        id_str => {
                            errors.push(syn::Error::new(
                                value.span(),
                                format!("unrecognized entry `{}`", id_str),
                            ));
                        }
                    }
                }

                if some_list.is_none() {
                    errors.push(syn::Error::new(span, "expected key `some_list` not found"));
                }
                if some_ident_list.is_none() {
                    errors.push(syn::Error::new(
                        span,
                        "expected key `some_ident_list` not found",
                    ));
                }
                if some_bool.is_none() {
                    errors.push(syn::Error::new(span, "expected key `some_bool` not found"));
                }

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

        #[derive(PartialEq, Debug)]
        pub struct SubAttribute {
            pub some_sub_bool: bool,
        }

        impl Attribute for SubAttribute {
            fn parse(values: Values, span: Span) -> Result<Self> {
                let mut errors = Vec::new();

                let mut some_sub_bool: Option<bool> = None;

                for value in values {
                    let id = match value.identifier() {
                        Some(id) => id,
                        None => continue,
                    };
                    match id.as_str() {
                        id_str if id_str == "some_sub_bool" => {
                            some_sub_bool.insert_value(id_str, value, &mut errors);
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
