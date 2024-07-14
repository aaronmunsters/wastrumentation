use num_integer::div_rem;

pub trait Alphabetical {
    /// Yield an alphabetic representation itself.
    fn to_alphabetic(&self) -> String;
}

impl Alphabetical for usize {
    /// The alphabetic representation of a `usize` is shown in
    /// base 26. The digit `0` is `a`, the digit `1` is `b`, etc.
    ///
    /// Since any numerical representation could be prefixed with
    /// zeroes (e.g. `42` and `000042` are the same number), we
    /// could do the same in alphabetical representation to prefix
    /// every representation with any number of `a`'s. As such,
    /// the value `0` will yield an `a` but all other values larger
    /// than `25` will start with a `b` or greater character.
    fn to_alphabetic(&self) -> String {
        if *self == 0 {
            String::from('a')
        } else {
            let mut result = String::new();
            let mut number = *self;
            while number != 0 {
                let (div, rem) = div_rem(number, 26);
                result.push(char::from(b'a' + u8::try_from(rem).unwrap()));
                number = div;
            }
            result.chars().rev().collect()
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_based_on_behavior() {
        let mut seen: HashSet<String> = HashSet::new();
        for input in 0..100_000 {
            let alphabetic = input.to_alphabetic();
            alphabetic
                .chars()
                .for_each(|c| assert!(c.is_ascii_alphabetic()));
            let new_insertion = seen.insert(alphabetic);
            assert!(new_insertion)
        }
    }

    #[test]
    fn test() {
        for (input, expectation) in [
            (0, "a"),
            (1, "b"),
            (2, "c"),
            // ... rest of the alphabet ...
            (23, "x"),
            (24, "y"),
            (25, "z"),
            (26, "ba"), // starting at bb, not aa FIXME: low priority
        ] {
            assert_eq!(input.to_alphabetic(), expectation)
        }
    }
}
