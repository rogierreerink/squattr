use syn::Error;

pub trait CombineErrorsExt: Sized {
    fn combine_errors(self) -> Option<Error>;
}

impl CombineErrorsExt for Vec<Error> {
    fn combine_errors(self) -> Option<Error> {
        let first = match self.get(0) {
            Some(first) => first.clone(),
            None => return None,
        };

        let rest = match self.get(1..) {
            Some(rest) => rest,
            None => return Some(first),
        };

        Some(rest.iter().fold(first, |mut acc, next| {
            acc.combine(next.clone());
            acc
        }))
    }
}
