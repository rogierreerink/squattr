use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{
    Data, DataStruct, DeriveInput, Error, Field, Fields, FieldsNamed, Ident, PathArguments,
    PathSegment, Result, Type, TypePath, parse2, punctuated, spanned::Spanned,
};

pub fn expand(input: TokenStream) -> Result<TokenStream> {
    let input = parse2::<DeriveInput>(input)?;
    let ident = input.ident;
    match input.data {
        Data::Struct(DataStruct { fields, .. }) => expand_struct(ident.clone(), fields),
        Data::Enum(_) => Err(Error::new(Span::call_site(), "enums are not supported")),
        Data::Union(_) => Err(Error::new(Span::call_site(), "unions are not supported")),
    }
}

fn expand_struct(ident: Ident, fields: Fields) -> Result<TokenStream> {
    match fields {
        Fields::Named(FieldsNamed { named, .. }) => expand_named_struct(ident, named.iter()),
        Fields::Unnamed(_) => {
            return Err(Error::new(
                Span::call_site(),
                "unnamed structs are not supported",
            ));
        }
        Fields::Unit => {
            return Err(Error::new(
                Span::call_site(),
                "unit structs are not supported",
            ));
        }
    }
}

fn expand_named_struct(ident: Ident, fields: punctuated::Iter<Field>) -> Result<TokenStream> {
    let mut variables = TokenStream::new();
    let mut match_arms = TokenStream::new();
    let mut required_checks = TokenStream::new();
    let mut struct_fields = TokenStream::new();
    let mut field_strs = TokenStream::new();

    for field in fields {
        let ident = field.ident.as_ref().unwrap();
        let ident_str = ident.to_string();
        let ty = &field.ty;

        field_strs.extend(quote! {
            #ident_str,
        });

        match_arms.extend(quote_spanned! {
            ty.span()=>
            id_str if id_str == #ident_str => {
                #ident.insert_value(id_str, value, &mut errors);
            }
        });

        if is_optional(ty) {
            variables.extend(quote! {
                let mut #ident: #ty = ::std::option::Option::None;
            });

            struct_fields.extend(quote! {
                #ident,
            })
        } else {
            variables.extend(quote! {
                let mut #ident: ::std::option::Option<#ty> = ::std::option::Option::None;
            });

            if is_boolean(ty) {
                struct_fields.extend(quote! {
                    #ident: #ident.unwrap_or_default(),
                });
            } else {
                let error_msg = format!("required key `{}` not found", ident);
                required_checks.extend(quote! {
                    if #ident.is_none() {
                        errors.push(::syn::Error::new(span, #error_msg));
                    };
                });

                struct_fields.extend(quote! {
                    #ident: #ident.expect("values existence has been confirmed"),
                });
            }
        }
    }

    Ok(quote! {
        #[automatically_derived]
        impl ::squattr::attribute::Attribute for #ident {
            fn from_values(values: ::squattr::ast::Values) -> ::syn::Result<Self> {
                use ::squattr::{errors::ErrorsExt, types::ValueStorageExt};

                #variables

                let span = values.span();
                let mut errors = ::std::vec::Vec::new();

                for value in values {
                    let id = match value.identifier() {
                        ::std::option::Option::Some(id) => id,
                        ::std::option::Option::None => {
                            errors.push(::syn::Error::new(
                                value.span(),
                                ::std::format!("expected an identifier"),
                            ));
                            continue;
                        },
                    };

                    match id.as_str() {
                        #match_arms

                        id_str => {
                            let dym = match ::squattr::dym::did_you_mean(
                                &[#field_strs],
                                id_str,
                            ) {
                                Some(best_match) => format!(", did you mean `{}`?", best_match),
                                None => "".into()
                            };

                            errors.push(::syn::Error::new(
                                value.span(),
                                ::std::format!("unrecognized key `{}`{}", id_str, dym),
                            ));
                        }
                    }
                }

                #required_checks

                if let ::std::option::Option::Some(error) = errors.combine() {
                    return Err(error);
                }

                Ok(Self {
                    #struct_fields
                })
            }
        }
    })
}

/// Determine wether a type is a `::std::option::Option` (i.e. may be omitted).
///
/// See [matches_type_path] for more info.
///
#[inline]
fn is_optional(ty: &Type) -> bool {
    matches_type_path(
        ty,
        &[
            PathSegment {
                ident: Ident::new("std", Span::call_site()),
                arguments: PathArguments::None,
            },
            PathSegment {
                ident: Ident::new("option", Span::call_site()),
                arguments: PathArguments::None,
            },
            PathSegment {
                ident: Ident::new("Option", Span::call_site()),
                arguments: PathArguments::None,
            },
        ],
    )
}

/// Determine wether a type is a `::std::primitive::bool`.
///
/// See [matches_type_path] for more info.
///
#[inline]
fn is_boolean(ty: &Type) -> bool {
    matches_type_path(
        ty,
        &[
            PathSegment {
                ident: Ident::new("std", Span::call_site()),
                arguments: PathArguments::None,
            },
            PathSegment {
                ident: Ident::new("primitive", Span::call_site()),
                arguments: PathArguments::None,
            },
            PathSegment {
                ident: Ident::new("bool", Span::call_site()),
                arguments: PathArguments::None,
            },
        ],
    )
}

/// Check wether a type matches the `expected` path segments.
///
/// From back to front, the given type needs to completely match at least part
/// of the `expected` path segments. This gives users some flexibility with
/// regard to their type imports (compared to just checking for `Option`, eg).
/// However, the type path cannot be renamed and the user must make sure that
/// they use the default path and not some other path with the same name.
///
#[inline]
fn matches_type_path(ty: &Type, expected: &[PathSegment]) -> bool {
    let ty_segments = match ty {
        Type::Path(TypePath { path, .. }) => &path.segments,
        _ => return false,
    };

    ty_segments
        .iter()
        .rev()
        .zip(expected.iter().rev())
        .all(|(expected_seg, ty_seg)| expected_seg.ident == ty_seg.ident)
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use colored::Colorize;
    use proc_macro2::TokenStream;
    use quote::quote;

    use super::expand;

    #[test]
    fn expand_named_struct() {
        let input = quote! {
            struct FooAttribute {
                bar: String,
                baz: Option<bool>,
                ban: bool,
            }
        };

        let expect = quote! {
            #[automatically_derived]
            impl ::squattr::attribute::Attribute for FooAttribute {
                fn from_values(values: ::squattr::ast::Values) -> ::syn::Result<Self> {
                    use ::squattr::{errors::ErrorsExt, types::ValueStorageExt};

                    let mut bar: ::std::option::Option<String> = ::std::option::Option::None;
                    let mut baz: Option<bool> = ::std::option::Option::None;
                    let mut ban: ::std::option::Option<bool> = ::std::option::Option::None;

                    let span = values.span();
                    let mut errors = ::std::vec::Vec::new();

                    for value in values {
                        let id = match value.identifier() {
                            ::std::option::Option::Some(id) => id,
                            ::std::option::Option::None => {
                                errors.push(::syn::Error::new(
                                    value.span(),
                                    ::std::format!("expected an identifier"),
                                ));
                                continue;
                            },
                        };
                        match id.as_str() {
                            id_str if id_str == "bar" => {
                                bar.insert_value(id_str, value, &mut errors);
                            }
                            id_str if id_str == "baz" => {
                                baz.insert_value(id_str, value, &mut errors);
                            }
                            id_str if id_str == "ban" => {
                                ban.insert_value(id_str, value, &mut errors);
                            }
                            id_str => {
                                let dym = match ::squattr::dym::did_you_mean(
                                    &["bar", "baz", "ban"],
                                    id_str,
                                ) {
                                    Some(best_match) => format!(", did you mean `{}`?", best_match),
                                    None => "".into(),
                                };

                                errors
                                    .push(
                                        ::syn::Error::new(
                                            value.span(),
                                        ::std::format!("unrecognized key `{}`{}", id_str, dym),
                                        ),
                                    );
                            }
                        }
                    }

                    if bar.is_none() {
                        errors.push(::syn::Error::new(span, "required key `bar` not found"));
                    }

                    if let ::std::option::Option::Some(error) = errors.combine() {
                        return Err(error);
                    }

                    Ok(Self {
                        bar: bar.expect("values existence has been confirmed"),
                        baz,
                        ban: ban.unwrap_or_default(),
                    })
                }
            }
        };

        let time_start = Instant::now();
        let expanded = expand(input).unwrap();
        let time_end = Instant::now();

        assert_eq_token_streams(&expanded, &expect);
        assess_expansion_duration(time_start, time_end, 500);
    }

    pub fn assert_eq_token_streams(a: &TokenStream, b: &TokenStream) {
        let a_str = a.to_string();
        let a_parsed = syn::parse_file(&a_str).unwrap();
        let a_pretty = prettyplease::unparse(&a_parsed);

        let b_str = b.to_string();
        let b_parsed = syn::parse_file(&b_str).unwrap();
        let b_pretty = prettyplease::unparse(&b_parsed);

        pretty_assertions::assert_eq!(a_pretty, b_pretty);
    }

    fn assess_expansion_duration(start: Instant, end: Instant, lt_us: u128) {
        let duration = (end - start).as_micros();
        let duration_str = format!("expansion duration: {}us", duration);
        if duration >= lt_us {
            println!("{}", duration_str.red(),);
        } else {
            println!("{}", duration_str.yellow(),);
        }
    }
}
