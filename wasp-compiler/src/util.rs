use num_integer::div_rem;

pub trait Alphabetical {
    fn to_alphabetic(&self) -> String;
}

impl Alphabetical for usize {
    fn to_alphabetic(&self) -> String {
        let mut result = String::new();
        let mut number = *self;
        while number > 25 {
            let (division, rem) = div_rem(number, 25);
            result.push(char::from(b'a' + rem as u8));
            number = division;
            // println!("{self}, {number}, {division}, {rem}, {result}")
        }
        result.push(char::from(b'a' + number as u8));
        result.chars().rev().collect::<String>()
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
            (26, "bb"), // starting at bb, not aa FIXME: low priority
        ] {
            assert_eq!(input.to_alphabetic(), expectation)
        }
    }
}
