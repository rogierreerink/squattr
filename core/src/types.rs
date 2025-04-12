use syn::{Error, Ident, Lit, Result};

use crate::{
    ast::{Expr, List, Value},
    errors::ErrorsExt,
};

impl ParseValue for bool {
    fn parse(value: Value) -> Result<Self> {
        match value {
            Value::Expr(Expr { value, .. }) => match value.as_ref() {
                Value::Lit(Lit::Bool(lit_bool)) => Ok(lit_bool.value()),
                value => Err(format_error(value, "a boolean (`true`, `false`)")),
            },
            Value::Ident(_) => Ok(true),
            value => Err(format_error(&value, "a boolean expression")),
        }
    }
}

macro_rules! impl_integers {
    ($( $t:ty ),*) => {
        $(impl ParseValue for $t {
            fn parse(value: Value) -> Result<Self> {
                match value {
                    Value::Expr(Expr { value, .. }) => match value.as_ref() {
                        Value::Lit(Lit::Int(lit_int)) => Ok(lit_int.base10_parse()?),
                        value => Err(format_error(value, "an integer")),
                    },
                    Value::Lit(Lit::Int(lit_int)) => Ok(lit_int.base10_parse()?),
                    value => Err(format_error(&value, "an integer")),
                }
            }
        })*
    };
}

impl_integers!(
    usize, u128, u64, u32, u16, u8, isize, i128, i64, i32, i16, i8
);

macro_rules! impl_floats {
    ($( $t:ty ),*) => {
        $(impl ParseValue for $t {
            fn parse(value: Value) -> Result<Self> {
                match value {
                    Value::Expr(Expr { value, .. }) => match value.as_ref() {
                        Value::Lit(Lit::Float(lit_float)) => Ok(lit_float.base10_parse()?),
                        Value::Lit(Lit::Int(lit_int)) => Ok(lit_int.base10_parse()?),
                        value => Err(format_error(value, "a decimal")),
                    },
                    Value::Lit(Lit::Float(lit_float)) => Ok(lit_float.base10_parse()?),
                    Value::Lit(Lit::Int(lit_int)) => Ok(lit_int.base10_parse()?),
                    value => Err(format_error(&value, "a decimal")),
                }
            }
        })*
    };
}

impl_floats!(f64, f32);

impl ParseValue for String {
    fn parse(value: Value) -> Result<Self> {
        match value {
            Value::Expr(Expr { value, .. }) => match value.as_ref() {
                Value::Lit(Lit::Str(lit_str)) => Ok(lit_str.value()),
                value => Err(format_error(value, "a string literal")),
            },
            Value::Lit(Lit::Str(lit_str)) => Ok(lit_str.value()),
            value => Err(format_error(&value, "a string literal")),
        }
    }
}

impl ParseValue for Vec<String> {
    fn parse(value: Value) -> Result<Self> {
        match value {
            Value::List(List { values, .. }) => {
                let mut errors = vec![];
                let mut strings = vec![];

                for value in values {
                    match value {
                        Value::Lit(Lit::Str(lit_str)) => strings.push(lit_str.value()),
                        value => errors.push(format_error(&value, "a string literal")),
                    }
                }

                if let Some(error) = errors.combine_errors() {
                    return Err(error);
                }

                Ok(strings)
            }
            value => Err(format_error(&value, "a list of string literals")),
        }
    }
}

impl ParseValue for Ident {
    fn parse(value: Value) -> Result<Self> {
        match value {
            Value::Ident(ident) => Ok(ident),
            value => Err(format_error(&value, "an identifier")),
        }
    }
}

impl ParseValue for Vec<Ident> {
    fn parse(value: Value) -> Result<Self> {
        match value {
            Value::List(List { values, .. }) => {
                let mut errors = vec![];
                let mut idents = vec![];

                for value in values {
                    match value {
                        Value::Ident(ident) => idents.push(ident),
                        value => errors.push(format_error(&value, "an identifier")),
                    }
                }

                if let Some(error) = errors.combine_errors() {
                    return Err(error);
                }

                Ok(idents)
            }
            value => Err(format_error(&value, "a list of identifiers")),
        }
    }
}

impl ParseValue for Lit {
    fn parse(value: Value) -> Result<Self> {
        match value {
            Value::Expr(Expr { value, .. }) => match value.as_ref() {
                Value::Lit(lit) => Ok(lit.clone()),
                value => Err(format_error(value, "a literal")),
            },
            value => Err(format_error(&value, "a literal expression")),
        }
    }
}

#[inline]
pub fn format_error(value: &Value, expect: &str) -> Error {
    Error::new(
        value.span(),
        match value.identifier() {
            Some(id) => format!("`{}` expects {}", id, expect),
            None => format!("expected {}", expect),
        },
    )
}

pub trait ParseValue: Sized {
    fn parse(value: Value) -> Result<Self>;
}

pub trait ParseValueExt: Sized {
    fn parse<T: ParseValue>(self) -> Result<T>;
}

impl ParseValueExt for Value {
    fn parse<T: ParseValue>(self) -> Result<T> {
        T::parse(self)
    }
}

pub trait ValueStorageExt: Sized {
    fn insert_value(&mut self, id: &str, value: Value, errors: &mut Vec<Error>);
    fn append_value(&mut self, id: &str, value: Value, errors: &mut Vec<Error>);
}

impl<T> ValueStorageExt for Option<T>
where
    T: ParseValue,
{
    fn insert_value(&mut self, id: &str, value: Value, errors: &mut Vec<Error>) {
        if self.is_some() {
            errors.push(Error::new(
                value.span(),
                format!("duplicate entry for `{}`", id),
            ));
        } else {
            match value.parse() {
                Ok(value) => {
                    self.replace(value);
                }
                Err(error) => {
                    errors.push(error);
                }
            }
        }
    }

    fn append_value(&mut self, id: &str, value: Value, errors: &mut Vec<Error>) {
        errors.push(Error::new(
            value.span(),
            format!("cannot append multiple entries for `{}`", id),
        ));
    }
}
