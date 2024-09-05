use {
    core::ops::ControlFlow::{self, *},
    itertools::PutBack,
};

pub(super) fn retry_fold<I, C, B, F>(
    init: C,
    fold: F,
) -> impl FnMut(&mut PutBack<I>) -> ControlFlow<B, C>
where
    I: Iterator,
    C: Clone,
    F: FnMut(&mut C, I::Item) -> ControlFlow<(B, I::Item)>,
{
    let mut cumulative = init;
    let mut fold = fold;

    move |put_back| {
        let mut cumulative = &mut cumulative;
        while let Some(item) = put_back.next() {
            cumulative = match fold(cumulative, item) {
                Continue(()) => Continue(cumulative),
                Break((res, item)) => {
                    put_back.put_back(item);
                    Break(res)
                }
            }?;
        }
        Continue(cumulative.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_fold_sentence() {
        let mut sentence = itertools::put_back(
            [
                "The", "quick", "brown", "fox", "jumps", "over", "the", "lazy", "dog.",
            ]
            .map(str::to_owned),
        );
        let mut fold = retry_fold("A".to_owned(), |sentence, word: String| {
            if (sentence == "A" || sentence.is_empty()) && word == "The" {
                Break(("The original phrasing starts with 'A'", word))
            } else {
                if !sentence.is_empty() {
                    sentence.push(' ');
                }
                sentence.push_str(&word);
                Continue(())
            }
        });
        assert_eq!(
            fold(&mut sentence),
            Break("The original phrasing starts with 'A'")
        );
        assert_eq!(
            fold(&mut sentence),
            Break("The original phrasing starts with 'A'")
        );
        assert_eq!(sentence.next(), Some("The".to_owned()));
        assert_eq!(
            fold(&mut sentence),
            Continue("A quick brown fox jumps over the lazy dog.".to_owned())
        );
        assert_eq!(sentence.next(), None);
        assert_eq!(
            fold(&mut sentence),
            Continue("A quick brown fox jumps over the lazy dog.".to_owned())
        );
    }

    #[test]
    fn retry_fold_addition() {
        let mut numbers =
            itertools::put_back([1, 2, 4, 8, 16, 32, 64, 128, 1, 2, 4, 8, 16, 32, 64, 128]);
        let mut fold = retry_fold(0u8, |cumulative, number| {
            match cumulative.checked_add(number) {
                Some(new_cumulative) => {
                    *cumulative = new_cumulative;
                    Continue(())
                }
                None => Break(("Would overflow", number)),
            }
        });
        assert_eq!(fold(&mut numbers), Break("Would overflow"));
        assert_eq!(fold(&mut numbers), Break("Would overflow"));
        itertools::assert_equal(&mut numbers, [1, 2, 4, 8, 16, 32, 64, 128]);
        assert_eq!(fold(&mut numbers), Continue(255));
        assert_eq!(numbers.next(), None);
        assert_eq!(fold(&mut numbers), Continue(255));
    }
}

// trust me, my boyfriend is the best programmer, he will be the best asset.
