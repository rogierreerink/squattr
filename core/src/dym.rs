use strsim::jaro_winkler;

pub fn did_you_mean(options: &[&'static str], given: &str) -> Option<String> {
    let mut current_best_match = options
        .get(0)
        .map(|&option| (option, jaro_winkler(option, given)));

    if let Some(more) = options.get(1..) {
        for option in more {
            let score = jaro_winkler(option, given);

            if current_best_match
                .is_some_and(|(_, current_highest_score)| score > current_highest_score)
            {
                current_best_match = Some((option, score))
            }
        }
    }

    if let Some((option, score)) = current_best_match {
        if score > 0.8 {
            return Some(option.into());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use crate::dym::did_you_mean;

    #[test]
    fn found_match() {
        let options = ["hola, mundo", "hello, world", "foobar"];
        let input = "hllo wrd";

        assert_eq!(
            "hello, world",
            did_you_mean(&options, input).expect("must have a best match")
        );
    }

    #[test]
    fn found_no_match() {
        let options = ["hola, mundo", "hello, world", "foobar"];
        let input = "hi munbar";

        assert_eq!(None, did_you_mean(&options, input));
    }
}
