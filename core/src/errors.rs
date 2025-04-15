use syn::Error;

pub trait ErrorsExt: Sized {
    fn combine(self) -> Option<Error>;
}

impl ErrorsExt for Vec<Error> {
    fn combine(self) -> Option<Error> {
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
