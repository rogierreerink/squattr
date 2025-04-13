use syn::{Error, Ident, Lit, LitBool, LitFloat, LitInt, LitStr, Result};

use crate::{
    ast::{Expr, List, Value},
    errors::ErrorsExt,
};

macro_rules! impl_integers {
    ($( $t:ty ),*) => {
        $(impl ParseValue for $t {
            fn parse(value: Value) -> Result<Self> {
                match value {
                    Value::Expr(Expr { value, .. }) => match value.as_ref() {
                        Value::Lit(Lit::Int(lit_int)) => Ok(lit_int.base10_parse()?),
                        value => Err(format_error(value, "integer")),
                    },
                    Value::Lit(Lit::Int(lit_int)) => Ok(lit_int.base10_parse()?),
                    value => Err(format_error(&value, "integer")),
                }
            }
        }

        impl ParseValue for Vec<$t> {
            fn parse(value: Value) -> Result<Self> {
                match value {
                    Value::List(List { values, .. }) => {
                        let mut errors = vec![];
                        let mut lits = vec![];

                        for value in values {
                            match value {
                                Value::Lit(Lit::Float(lit_float)) => lits.push(lit_float.base10_parse()?),
                                Value::Lit(Lit::Int(lit_int)) => lits.push(lit_int.base10_parse()?),
                                value => errors.push(format_error(&value, "decimal")),
                            }
                        }

                        if let Some(error) = errors.combine_errors() {
                            return Err(error);
                        }

                        Ok(lits)
                    }
                    value => Err(format_error(&value, "list of decimals")),
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
                        value => Err(format_error(value, "decimal")),
                    },
                    Value::Lit(Lit::Float(lit_float)) => Ok(lit_float.base10_parse()?),
                    Value::Lit(Lit::Int(lit_int)) => Ok(lit_int.base10_parse()?),
                    value => Err(format_error(&value, "decimal")),
                }
            }
        }

        impl ParseValue for Vec<$t> {
            fn parse(value: Value) -> Result<Self> {
                match value {
                    Value::List(List { values, .. }) => {
                        let mut errors = vec![];
                        let mut lits = vec![];

                        for value in values {
                            match value {
                                Value::Lit(Lit::Float(lit_float)) => lits.push(lit_float.base10_parse()?),
                                Value::Lit(Lit::Int(lit_int)) => lits.push(lit_int.base10_parse()?),
                                value => errors.push(format_error(&value, "decimal")),
                            }
                        }

                        if let Some(error) = errors.combine_errors() {
                            return Err(error);
                        }

                        Ok(lits)
                    }
                    value => Err(format_error(&value, "list of decimals")),
                }
            }
        })*
    };
}

impl_floats!(f64, f32);

impl ParseValue for bool {
    fn parse(value: Value) -> Result<Self> {
        match value {
            Value::Expr(Expr { value, .. }) => match value.as_ref() {
                Value::Lit(Lit::Bool(lit_bool)) => Ok(lit_bool.value()),
                value => Err(format_error(value, "boolean (`true`, `false`)")),
            },
            Value::Ident(_) => Ok(true),
            value => Err(format_error(&value, "boolean expression")),
        }
    }
}

impl ParseValue for String {
    fn parse(value: Value) -> Result<Self> {
        match value {
            Value::Expr(Expr { value, .. }) => match value.as_ref() {
                Value::Lit(Lit::Str(lit_str)) => Ok(lit_str.value()),
                value => Err(format_error(value, "string literal")),
            },
            value => Err(format_error(&value, "string literal")),
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
                        value => errors.push(format_error(&value, "string literal")),
                    }
                }

                if let Some(error) = errors.combine_errors() {
                    return Err(error);
                }

                Ok(strings)
            }
            value => Err(format_error(&value, "list of string literals")),
        }
    }
}

impl ParseValue for Ident {
    fn parse(value: Value) -> Result<Self> {
        match value {
            Value::Ident(ident) => Ok(ident),
            value => Err(format_error(&value, "identifier")),
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
                        value => errors.push(format_error(&value, "identifier")),
                    }
                }

                if let Some(error) = errors.combine_errors() {
                    return Err(error);
                }

                Ok(idents)
            }
            value => Err(format_error(&value, "list of identifiers")),
        }
    }
}

impl ParseValue for Lit {
    fn parse(value: Value) -> Result<Self> {
        match value {
            Value::Expr(Expr { value, .. }) => match value.as_ref() {
                Value::Lit(lit) => Ok(lit.clone()),
                value => Err(format_error(value, "literal")),
            },
            value => Err(format_error(&value, "literal expression")),
        }
    }
}

impl ParseValue for Vec<Lit> {
    fn parse(value: Value) -> Result<Self> {
        match value {
            Value::List(List { values, .. }) => {
                let mut errors = vec![];
                let mut lits = vec![];

                for value in values {
                    match value {
                        Value::Lit(lit) => lits.push(lit),
                        value => errors.push(format_error(&value, "literal")),
                    }
                }

                if let Some(error) = errors.combine_errors() {
                    return Err(error);
                }

                Ok(lits)
            }
            value => Err(format_error(&value, "list of literals")),
        }
    }
}

macro_rules! impl_lit_variants {
    ($( ($t:ty, $e:path, $x:literal, $xp:literal) ),*) => {
        $(impl ParseValue for $t {
            fn parse(value: Value) -> Result<Self> {
                match value {
                    Value::Expr(Expr { value, .. }) => match value.as_ref() {
                        Value::Lit($e(lit)) => Ok(lit.clone()),
                        value => Err(format_error(value, $x)),
                    },
                    value => Err(format_error(&value, concat!($x, " expression"))),
                }
            }
        }

        impl ParseValue for Vec<$t> {
            fn parse(value: Value) -> Result<Self> {
                match value {
                    Value::List(List { values, .. }) => {
                        let mut errors = vec![];
                        let mut lits = vec![];

                        for value in values {
                            match value {
                                Value::Lit($e(lit)) => lits.push(lit),
                                value => errors.push(format_error(&value, $x)),
                            }
                        }

                        if let Some(error) = errors.combine_errors() {
                            return Err(error);
                        }

                        Ok(lits)
                    }
                    value => Err(format_error(&value, concat!("list of ", $xp))),
                }
            }
        })*
    }
}

impl_lit_variants!(
    (LitBool, Lit::Bool, "boolean", "booleans"),
    (LitFloat, Lit::Float, "decimal", "decimals"),
    (LitInt, Lit::Int, "integer", "integers"),
    (LitStr, Lit::Str, "string literal", "string literals")
);

/// Create a type conversion error.
///
#[inline]
pub fn format_error(value: &Value, expect: &str) -> Error {
    Error::new(
        value.span(),
        match value.identifier() {
            Some(id) => format!("expected {} (`{}`)", expect, id),
            None => format!("{} expected", expect),
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
        if !self.is_some() {
            match value.parse() {
                Ok(value) => {
                    self.replace(value);
                }
                Err(error) => {
                    errors.push(error);
                }
            }
        } else {
            errors.push(Error::new(
                value.span(),
                format!("duplicate entry for `{}`", id),
            ));
        }
    }

    fn append_value(&mut self, id: &str, value: Value, errors: &mut Vec<Error>) {
        errors.push(Error::new(
            value.span(),
            format!("cannot append multiple entries for `{}`", id),
        ));
    }
}
