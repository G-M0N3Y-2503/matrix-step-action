use {
    super::*,
    core::fmt::Debug,
    itertools::Itertools,
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
};

#[cfg(test)]
macro_rules! to_string {
    ($string:expr) => {
        $string.to_string()
    };
    [$($string:expr),+] => {
        vec![$(to_string!($string)),+]
    };
}

#[cfg(test)]
macro_rules! test_serde {
    ($type:ty, $tests:expr) => {
        for (i, (rust, json)) in $tests.into_iter().enumerate() {
            assert_eq!(
                serde_json::to_value(&rust).unwrap_or_else(|err| panic!(
                    "test case {i} did not seralise to a json value: {err}"
                )),
                json,
                "test case {i} did not match the expected json value"
            );
            assert_eq!(
                serde_json::from_value::<$type>(json).unwrap_or_else(|err| panic!(
                    "test case {i} did not deserialise to rust: {err}"
                )),
                rust,
                "test case {i} did not match the expected structure"
            );
        }
    };
}

mod filter;
pub use filter::Filter;

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValuesFrom {
    Values(Vec<String>),
    Combinations {
        of: usize,
        from: Box<ValuesFrom>,
        join_with: String,
    },
    Select {
        from: Box<ValuesFrom>,
        r#where: filter::Filter,
    },
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct EnvMatrix(HashMap<String, ValuesFrom>);

impl FromIterator<(String, ValuesFrom)> for EnvMatrix {
    fn from_iter<T: IntoIterator<Item = (String, ValuesFrom)>>(iter: T) -> Self {
        Self(HashMap::from_iter(iter))
    }
}

impl IntoIterator for EnvMatrix {
    type Item = HashMap<String, String>;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            var_iters: HashMap::from_iter(self.0.into_iter().map(|(k, v)| (k, v.into_iter()))),
        }
    }
}

pub struct IntoIter {
    var_iters: HashMap<String, Box<dyn Iterator<Item = String>>>,
}

impl Iterator for IntoIter {
    type Item = HashMap<String, String>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(HashMap::from_iter(
            self.var_iters
                .iter_mut()
                .filter_map(|(k, v)| v.next().map(|v| (k.clone(), v))),
        ))
    }
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
            ValuesFrom::Select { from, r#where } => Box::new(
                from.into_iter(), // .filter(move |value| (filter::CompareContext::from(&r#where))(value)),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use {
        super::{filter::FilterContext::*, filter::ValueComparer::*, *},
        serde_json::json,
        wasm_bindgen_test::{console_log, wasm_bindgen_test},
    };

    #[test]
    #[wasm_bindgen_test]
    fn test_json() {
        test_serde!(
            ValuesFrom,
            [
                (
                    ValuesFrom::Values(to_string![1, 2, 3]),
                    json!({"values": ["1", "2", "3"]}),
                ),
                (
                    ValuesFrom::Combinations {
                        of: 2,
                        from: Box::new(ValuesFrom::Values(to_string![1, 2, 3])),
                        join_with: to_string!(", "),
                    },
                    json!({"combinations": {
                        "of": 2,
                        "from": {"values" :["1", "2", "3"]},
                        "join_with": ", "
                    }}),
                ),
                (
                    ValuesFrom::Select {
                        from: Box::new(ValuesFrom::Values(to_string![1, 2, 3])),
                        r#where: Filter::Context(Value(Equals(to_string!(1)))),
                    },
                    json!({"select": {
                        "from": {"values": ["1", "2", "3"]},
                        "where": {"value": {"equals": "1"}}
                    }}),
                ),
            ]
        )
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_iter() {
        let values = vec!["1".to_owned(), "2".to_owned(), "3".to_owned()];

        itertools::assert_equal(
            ValuesFrom::Values(values.clone()).into_iter(),
            values.clone(),
        );

        itertools::assert_equal(
            ValuesFrom::Combinations {
                of: 2,
                from: Box::new(ValuesFrom::Values(values.clone())),
                join_with: ", ".to_owned(),
            },
            vec!["1, 2", "1, 3", "2, 3"],
        );
        itertools::assert_equal(
            ValuesFrom::Combinations {
                of: 2,
                from: Box::new(ValuesFrom::Combinations {
                    of: 2,
                    from: Box::new(ValuesFrom::Values(values.clone())),
                    join_with: ", ".to_owned(),
                }),
                join_with: " - ".to_owned(),
            },
            vec!["1, 2 - 1, 3", "1, 2 - 2, 3", "1, 3 - 2, 3"],
        );
    }
}
