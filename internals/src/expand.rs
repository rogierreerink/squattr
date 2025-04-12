use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    Data, DataStruct, DeriveInput, Error, Field, Fields, FieldsNamed, Ident, PathArguments,
    PathSegment, Result, Type, TypePath, parse2, punctuated,
};

pub fn expand(input: TokenStream) -> Result<TokenStream> {
    let input = parse2::<DeriveInput>(input)?;
    match input.data {
        Data::Struct(DataStruct { fields, .. }) => expand_struct(input.ident, fields),
        Data::Enum(_) => return Err(Error::new(Span::call_site(), "enums are not supported")),
        Data::Union(_) => return Err(Error::new(Span::call_site(), "unions are not supported")),
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

    for field in fields {
        let ident = field.ident.as_ref().unwrap();
        let ident_str = ident.to_string();
        let ty = &field.ty;

        match_arms.extend(quote! {
            id_str if id_str == #ident_str => {
                #ident.from_value(id_str, value, &mut errors);
            }
        });

        if is_option_type(&field.ty) {
            variables.extend(quote! {
                let mut #ident: #ty = None;
            });

            struct_fields.extend(quote! {
                #ident: #ident.unwrap_or_default(),
            })
        } else {
            variables.extend(quote! {
                let mut #ident: Option<#ty> = None;
            });

            let error_msg = format!("expected key `{}` not found", ident);
            required_checks.extend(quote! {
                if #ident.is_none() {
                    errors.push(syn::Error::new(span, #error_msg));
                };
            });

            struct_fields.extend(quote! {
                #ident,
            });
        }
    }

    Ok(quote! {
        impl TryFrom<(Values, Span)> for #ident {
            type Error = syn::Error;

            fn try_from((values, span): (Values, Span)) -> syn::Result<Self> {
                let mut errors = Vec::new();

                #variables

                for value in values {
                    let id = match value.identifier() {
                        Some(id) => id,
                        None => continue,
                    };

                    match id.as_str() {
                        #match_arms

                        id_str => {
                            errors.push(syn::Error::new(
                                value.span(),
                                format!("unrecognized entry `{}`", id_str),
                            ));
                        }
                    }
                }

                #required_checks

                if let Some(error) = errors.combine_errors() {
                    return Err(error);
                }

                Ok(Self {
                    #struct_fields
                })
            }
        }
    })
}

/// Determine wether a type is wrapped in an Option.
///
/// From back to front, the given type needs to completely follow at least part
/// of the `std::option::Option` type path. This gives users some flexibility
/// with regard to their type imports (comparted to just checking for `Option`).
/// However, the `Option` type path cannot be renamed.
///
fn is_option_type(ty: &Type) -> bool {
    let option_path_segments = [
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
    ];

    let ty_path_segments = match ty {
        Type::Path(TypePath { path, .. }) => &path.segments,
        _ => return false,
    };

    ty_path_segments
        .iter()
        .rev()
        .zip(option_path_segments.iter().rev())
        .all(|(ty_path_segment, option_path_segment)| {
            ty_path_segment.ident == option_path_segment.ident
        })
}

#[cfg(test)]
mod tests {
    use quote::quote;

    use crate::tests::assert_eq_token_streams;

    use super::expand;

    #[test]
    fn expand_named_struct() {
        let input = quote! {
            struct FooAttributes {
                bar: String,
                baz: Option<bool>,
            }
        };

        let expect = quote! {
            impl TryFrom<(Values, Span)> for FooAttributes {
                type Error = syn::Error;
                fn try_from((values, span): (Values, Span)) -> syn::Result<Self> {
                    let mut errors = Vec::new();
                    let mut bar: Option<String> = None;
                    let mut baz: Option<bool> = None;
                    for value in values {
                        let id = match value.identifier() {
                            Some(id) => id,
                            None => continue,
                        };
                        match id.as_str() {
                            id_str if id_str == "bar" => {
                                bar.from_value(id_str, value, &mut errors);
                            }
                            id_str if id_str == "baz" => {
                                baz.from_value(id_str, value, &mut errors);
                            }
                            id_str => {
                                errors
                                    .push(
                                        syn::Error::new(
                                            value.span(),
                                            format!("unrecognized entry `{}`", id_str),
                                        ),
                                    );
                            }
                        }
                    }
                    if bar.is_none() {
                        errors.push(syn::Error::new(span, "expected key `bar` not found"));
                    }
                    if let Some(error) = errors.combine_errors() {
                        return Err(error);
                    }
                    Ok(Self {
                        bar,
                        baz: baz.unwrap_or_default(),
                    })
                }
            }
        };

        assert_eq_token_streams(&expand(input).unwrap(), &expect);
    }
}
