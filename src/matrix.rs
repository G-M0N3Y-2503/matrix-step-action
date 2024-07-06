use {
    super::*,
    serde::{Deserialize, Serialize},
};

mod values_from {
    use {super::*, itertools::Itertools};

    #[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
    pub enum ValuesFrom {
        Values(Vec<String>),
        Combinations {
            of: usize,
            from: Box<ValuesFrom>,
            join_with: String,
        },
    }

    impl IntoIterator for ValuesFrom {
        type Item = String;
        type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

        fn into_iter(self) -> Self::IntoIter {
            match self {
                ValuesFrom::Values(values) => Box::new(values.into_iter()),
                ValuesFrom::Combinations {
                    of,
                    from,
                    join_with,
                } => Box::new(
                    from.into_iter()
                        .combinations(of)
                        .map(move |c| c.join(&join_with)),
                ),
            }
        }
    }
}
pub use values_from::ValuesFrom;

mod env_vars {
    use {super::*, std::collections::HashMap};

    #[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
    pub struct EnvVars(HashMap<String, ValuesFrom>);

    impl IntoIterator for EnvVars {
        type Item;
        type IntoIter;

        fn into_iter(self) -> Self::IntoIter {
            todo!()
        }
        // pub fn iter(&self) -> Iter {
        //     Iter {
        //         var_iters: HashMap::from_iter(self.0.iter().map(|(key, iter)| {
        //             (
        //                 key.as_str(),
        //                 match iter {
        //                     _ => Box::new(vec![].iter()),
        //                 },
        //             )
        //         })),
        //     }
        // }
    }

    impl FromIterator<(String, ValuesFrom)> for EnvVars {
        fn from_iter<T: IntoIterator<Item = (String, ValuesFrom)>>(iter: T) -> Self {
            Self(HashMap::from_iter(iter))
        }
    }

    pub struct Iter<'a> {
        var_iters: HashMap<&'a str, Box<dyn Iterator<Item = &'a str>>>,
    }
}
pub use env_vars::EnvVars;
