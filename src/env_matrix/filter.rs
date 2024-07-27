use {
    super::*,
    core::{
        fmt::Display,
        ops::{ControlFlow, Not},
    },
    log::*,
    std::collections::HashMap,
};

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Filter {
    All(Vec<Filter>),
    Any(Vec<Filter>),
    Not(Box<Filter>),
    #[serde(untagged)]
    Context(FilterContext),
}
impl Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
    }
}
impl<'filter> From<&'filter Filter> for CompareContext<'filter> {
    fn from(filter: &'filter Filter) -> Self {
        match filter {
            Filter::All(filters) => Box::new(|value, env| {
                match filters
                    .iter()
                    .map(Self::from)
                    .try_fold(Err("unused"), |_, compare| match compare(value, env) {
                        res @ Ok(true) => ControlFlow::Continue(res),
                        res @ Ok(false) | res @ Err(_) => ControlFlow::Break(res),
                    }) {
                    ControlFlow::Continue(res) | ControlFlow::Break(res) => res,
                }
            }),
            Filter::Any(filters) => Box::new(|value, env| {
                match filters
                    .iter()
                    .map(Self::from)
                    .try_fold(Err("unused"), |_, compare| match compare(value, env) {
                        res @ Ok(false) => ControlFlow::Continue(res),
                        res @ Ok(true) | res @ Err(_) => ControlFlow::Break(res),
                    }) {
                    ControlFlow::Continue(res) | ControlFlow::Break(res) => res,
                }
            }),
            parrent_filter @ Filter::Not(filter) => Box::new(move |value, env| {
                match filter.as_ref() {
                    // match Filter::Context to avoid wrapping in an unnecessary boxed closure
                    Filter::Context(context) => match ContextComparer::from(context) {
                        ContextComparer::Env(compare) => compare(env),
                        ContextComparer::Value(compare) => Ok(compare(value)),
                    },
                    filter @ Filter::Not(_) => {
                        warn!("Double negative found at:\n{parrent_filter}");
                        (Self::from(filter))(value, env)
                    }
                    filter => (Self::from(filter))(value, env),
                }
                .map(Not::not)
            }),
            Filter::Context(context) => {
                Box::new(move |value, env| match ContextComparer::from(context) {
                    ContextComparer::Env(compare) => compare(env),
                    ContextComparer::Value(compare) => Ok(compare(value)),
                })
            }
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterContext {
    #[serde(with = "filter_context_env_serde")]
    Env(Vec<(String, ValueComparer)>),
    Value(ValueComparer),
}
impl Display for FilterContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
    }
}

mod filter_context_env_serde {
    use {
        super::*,
        serde::{
            de::{MapAccess, Visitor},
            ser::SerializeMap,
            Deserializer, Serializer,
        },
    };

    pub fn serialize<S>(env: &[(String, ValueComparer)], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(env.len()))?;
        for (variable, value) in env {
            map.serialize_entry(variable, value)?;
        }
        map.end()
    }

    struct EnvVisitor;
    impl<'de> Visitor<'de> for EnvVisitor {
        type Value = Vec<(String, ValueComparer)>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("Environment Variable Map")
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut vec = Vec::with_capacity(map.size_hint().unwrap_or(0));
            while let Some((key, value)) = map.next_entry()? {
                vec.push((key, value));
            }
            Ok(vec)
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<(String, ValueComparer)>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(EnvVisitor)
    }
}

pub type CompareContext<'comparer> =
    Box<dyn Fn(&str, &HashMap<String, String>) -> Result<bool, &'comparer str> + 'comparer>;
type CompareValue<'comparer> = Box<dyn Fn(&str) -> bool + 'comparer>;
type CompareEnv<'comparer> =
    Box<dyn Fn(&HashMap<String, String>) -> Result<bool, &'comparer str> + 'comparer>;
enum ContextComparer<'comparer> {
    Env(CompareEnv<'comparer>),
    Value(CompareValue<'comparer>),
}
impl<'comparer> From<&'comparer FilterContext> for ContextComparer<'comparer> {
    fn from(context: &'comparer FilterContext) -> Self {
        match context {
            FilterContext::Value(comparer) => ContextComparer::Value(comparer.into()),
            FilterContext::Env(comparer) => ContextComparer::Env(Box::new(|env| {
                match comparer
                    .iter()
                    .try_fold(Err("unused"), |_, (env_var, comparer)| {
                        match env
                            .get(env_var)
                            .map(|value| (CompareValue::from(comparer))(value))
                        {
                            Some(true) => ControlFlow::Continue(Ok(true)),
                            Some(false) => ControlFlow::Break(Ok(false)),
                            None => ControlFlow::Break(Err(env_var.as_str())),
                        }
                    }) {
                    ControlFlow::Continue(res) | ControlFlow::Break(res) => res,
                }
            })),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueComparer {
    Equals(String),
    Contains(String),
}
impl Display for ValueComparer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
    }
}
impl<'comparer> From<&'comparer ValueComparer> for CompareValue<'comparer> {
    fn from(comparer: &'comparer ValueComparer) -> Self {
        match comparer {
            ValueComparer::Equals(other) => Box::new(|value| value.eq(other)),
            ValueComparer::Contains(other) => Box::new(move |value| value.contains(other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use {
        super::{Filter::*, FilterContext::*, ValueComparer::*, *},
        serde_json::json,
        wasm_bindgen_test::{console_log, wasm_bindgen_test},
    };

    #[test]
    #[wasm_bindgen_test]
    fn test_json() {
        test_serde!(
            Filter,
            [
                (
                    Filter::Context(Value(Equals(to_string!(1)))),
                    json!({"value" :{"equals": "1"}}),
                ),
                (
                    Filter::Context(Env(vec![(to_string!("VAR1"), Equals(to_string!(1))),])),
                    json!({"env" :{"VAR1": {"equals": "1"}}}),
                ),
                (
                    Filter::Context(Value(Contains(to_string!("on")))),
                    json!({"value" :{"contains": "on"}}),
                ),
                (
                    Filter::Any(vec![
                        Filter::Context(Value(Equals(to_string!(1)))),
                        Filter::Context(Value(Equals(to_string!(3)))),
                    ]),
                    json!({"any": [
                        {"value": {"equals": "1"}},
                        {"value": {"equals": "3"}}
                    ]}),
                ),
                (
                    Filter::Not(Box::new(Filter::Context(Value(Equals(to_string!(2)))))),
                    json!({"not": {"value": {"equals": "2"}}}),
                ),
                (
                    Filter::All(vec![
                        Filter::Context(Value(Equals(to_string!(1)))),
                        Filter::Not(Box::new(Filter::Any(vec![
                            Filter::Context(Value(Equals(to_string!(2)))),
                            Filter::Context(Value(Equals(to_string!(3)))),
                        ]))),
                    ]),
                    json!({"all" :[
                        {"value" :{"equals": "1"}},
                        {"not": {"any" :[
                            {"value": {"equals": "2"}},
                            {"value": {"equals": "3"}}
                        ]}}
                    ]}),
                ),
            ]
        )
    }

    #[test]
    #[wasm_bindgen_test]
    fn compare_value_test() {
        // Equals
        {
            assert!(CompareValue::from(&Equals(to_string!("one")))("one"));
            assert!(!CompareValue::from(&Equals(to_string!("on")))("one"));
        }

        // Contains
        {
            assert!(CompareValue::from(&Contains(to_string!("one")))("one"));
            assert!(CompareValue::from(&Contains(to_string!("on")))("one"));
            assert!(CompareValue::from(&Contains(to_string!("o")))("one"));
            assert!(CompareValue::from(&Contains(to_string!("n")))("one"));
            assert!(CompareValue::from(&Contains(to_string!("e")))("one"));
            assert!(!CompareValue::from(&Contains(to_string!("z")))("one"));
        }
    }

    macro_rules! to_env {
        [$(($variable:expr, $value:expr)),+] => {
            HashMap::from_iter([$((to_string!($variable), to_string!($value))),+])
        };
    }

    #[test]
    #[wasm_bindgen_test]
    fn compare_context_test() {
        macro_rules! unwrap_match {
            ($expression:expr, $pattern:pat $(if $guard:expr)? => $bound:ident $(,)?) => {
                match $expression {
                    $pattern $(if $guard)? => $bound,
                    _ => panic!("Does Not Match")
                }
            };
        }

        // Value
        {
            let filter = Value(Equals(to_string!("one")));
            let compare_value_eq_one = unwrap_match!(
                ContextComparer::from(&filter),
                ContextComparer::Value(compare) => compare,
            );

            assert!(compare_value_eq_one("one"));
            assert!(compare_value_eq_one("one"));
            assert!(!compare_value_eq_one("two"));
        }

        // Env
        {
            let filter = Env(vec![(to_string!("VAR1"), Equals(to_string!("one")))]);
            let compare_var1_eq_one = unwrap_match!(
                ContextComparer::from(&filter),
                ContextComparer::Env(compare) => compare,
            );

            assert!(matches!(
                compare_var1_eq_one(&to_env![("VAR1", "one")]),
                Ok(true)
            ));
            assert!(matches!(
                compare_var1_eq_one(&to_env![("VAR1", "one")]),
                Ok(true)
            ));
            assert!(matches!(
                compare_var1_eq_one(&to_env![("VAR1", "two")]),
                Ok(false)
            ));
            assert!(matches!(
                compare_var1_eq_one(&to_env![("VAR2", "one")]),
                Err("VAR1")
            ));
            assert!(matches!(compare_var1_eq_one(&HashMap::new()), Err("VAR1")));

            let filters = [
                Env(vec![
                    (to_string!("VAR1"), Equals(to_string!("one"))),
                    (to_string!("VAR3"), Equals(to_string!("three"))),
                ]),
                Env(vec![
                    (to_string!("VAR3"), Equals(to_string!("three"))),
                    (to_string!("VAR1"), Equals(to_string!("one"))),
                ]),
            ];
            for filter in filters {
                let compare_multipul = unwrap_match!(
                    ContextComparer::from(&filter),
                    ContextComparer::Env(compare) => compare,
                );
                assert!(matches!(
                    compare_multipul(&to_env![
                        ("VAR1", "one"),
                        ("VAR2", "two"),
                        ("VAR3", "three")
                    ]),
                    Ok(true)
                ));
                assert!(matches!(
                    compare_multipul(&to_env![
                        ("VAR3", "three"),
                        ("VAR2", "two"),
                        ("VAR1", "one")
                    ]),
                    Ok(true)
                ));
                assert!(matches!(
                    compare_multipul(&to_env![
                        ("VAR1", "one"),
                        ("VAR2", "irrelevant change"),
                        ("VAR3", "three")
                    ]),
                    Ok(true)
                ));
                assert!(matches!(
                    compare_multipul(&to_env![("VAR1", "one"), ("VAR3", "three")]),
                    Ok(true)
                ));
                assert!(matches!(
                    compare_multipul(&to_env![("VAR3", "three")]),
                    Err("VAR1")
                ));
                assert!(matches!(
                    compare_multipul(&to_env![("VAR1", "one")]),
                    Err("VAR3")
                ));
                assert!(matches!(
                    compare_multipul(&HashMap::new()),
                    Err("VAR1") | Err("VAR3") // not testing for order
                ));
                assert!(matches!(
                    compare_multipul(&to_env![
                        ("VAR1", "two"),
                        ("VAR2", "two"),
                        ("VAR3", "three")
                    ]),
                    Ok(false)
                ));
                assert!(matches!(
                    compare_multipul(&to_env![
                        ("VAR3", "three"),
                        ("VAR2", "two"),
                        ("VAR1", "two")
                    ]),
                    Ok(false)
                ));
                assert!(matches!(
                    compare_multipul(&to_env![("VAR1", "one"), ("VAR2", "two"), ("VAR3", "two")]),
                    Ok(false)
                ));
                assert!(matches!(
                    compare_multipul(&to_env![("VAR3", "two"), ("VAR2", "two"), ("VAR1", "one")]),
                    Ok(false)
                ));
                assert!(matches!(
                    compare_multipul(&to_env![("VAR1", "two"), ("VAR2", "two"), ("VAR3", "two")]),
                    Ok(false)
                ));
                assert!(matches!(
                    compare_multipul(&to_env![("VAR3", "two"), ("VAR2", "two"), ("VAR1", "two")]),
                    Ok(false)
                ));
            }
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn filter_test() {
        // Value
        {
            let value_eq_one = Filter::Context(Value(Equals(to_string!("one"))));
            let value_ne_one = Filter::Not(Box::new(value_eq_one.clone()));
            for filter in [&value_eq_one, &value_ne_one] {
                let compare_value = CompareContext::from(filter);
                assert!(
                    match compare_value("one", &to_env![("VAR1", "one"), ("VAR2", "two")]) {
                        Ok(true) if filter == &value_eq_one => true,
                        Ok(false) if filter == &value_ne_one => true,
                        matched => panic!("got {matched:?} for filter {filter}"),
                    }
                );
                assert!(
                    match compare_value("one", &to_env![("VAR3", "irrelevant change")]) {
                        Ok(true) if filter == &value_eq_one => true,
                        Ok(false) if filter == &value_ne_one => true,
                        matched => panic!("got {matched:?} for filter {filter}"),
                    }
                );
                assert!(
                    match compare_value("two", &to_env![("VAR1", "one"), ("VAR2", "two")]) {
                        Ok(false) if filter == &value_eq_one => true,
                        Ok(true) if filter == &value_ne_one => true,
                        matched => panic!("got {matched:?} for filter {filter}"),
                    }
                );
            }
        }

        // Env
        {
            let var1_eq_one =
                Filter::Context(Env(vec![(to_string!("VAR1"), Equals(to_string!("one")))]));
            let var1_ne_one = Filter::Not(Box::new(var1_eq_one.clone()));
            for filter in [&var1_eq_one, &var1_ne_one] {
                let compare_env = CompareContext::from(filter);
                assert!(
                    match compare_env("one", &to_env![("VAR1", "one"), ("VAR2", "two")]) {
                        Ok(true) if filter == &var1_eq_one => true,
                        Ok(false) if filter == &var1_ne_one => true,
                        matched => panic!("got {matched:?} for filter {filter}"),
                    }
                );
                assert!(match compare_env(
                    "irrelevant change",
                    &to_env![("VAR1", "one"), ("VAR2", "two")]
                ) {
                    Ok(true) if filter == &var1_eq_one => true,
                    Ok(false) if filter == &var1_ne_one => true,
                    matched => panic!("got {matched:?} for filter {filter}"),
                });
                assert!(match compare_env(
                    "one",
                    &to_env![("VAR1", "one"), ("VAR3", "irrelevant change")]
                ) {
                    Ok(true) if filter == &var1_eq_one => true,
                    Ok(false) if filter == &var1_ne_one => true,
                    matched => panic!("got {matched:?} for filter {filter}"),
                });
                assert!(
                    match compare_env("one", &to_env![("VAR1", "two"), ("VAR2", "two")]) {
                        Ok(false) if filter == &var1_eq_one => true,
                        Ok(true) if filter == &var1_ne_one => true,
                        matched => panic!("got {matched:?} for filter {filter}"),
                    }
                );
                assert!(match compare_env("one", &to_env![("VAR2", "two")]) {
                    Err("VAR1") if filter == &var1_eq_one => true,
                    Err("VAR1") if filter == &var1_ne_one => true,
                    matched => panic!("got {matched:?} for filter {filter}"),
                });
            }
        }

        // All
        {
            let var1_and_value_eq_one = Filter::All(vec![
                Context(Env(vec![(to_string!("VAR1"), Equals(to_string!("one")))])),
                Context(Value(Equals(to_string!("one")))),
            ]);
            let var1_and_value_ne_one = Filter::Not(Box::new(var1_and_value_eq_one.clone()));
            let var1_and_value_eq_one_reversed = Filter::All(vec![
                Context(Value(Equals(to_string!("one")))),
                Context(Env(vec![(to_string!("VAR1"), Equals(to_string!("one")))])),
            ]);
            let var1_and_value_ne_one_reversed =
                Filter::Not(Box::new(var1_and_value_eq_one_reversed.clone()));
            for filter in [
                &var1_and_value_eq_one,
                &var1_and_value_ne_one,
                &var1_and_value_eq_one_reversed,
                &var1_and_value_ne_one_reversed,
            ] {
                let compare_context = CompareContext::from(filter);
                assert!(
                    match compare_context("one", &to_env![("VAR1", "one"), ("VAR2", "two")]) {
                        Ok(true) if filter == &var1_and_value_eq_one => true,
                        Ok(false) if filter == &var1_and_value_ne_one => true,
                        Ok(true) if filter == &var1_and_value_eq_one_reversed => true,
                        Ok(false) if filter == &var1_and_value_ne_one_reversed => true,
                        matched => panic!("got {matched:?} for filter {filter}"),
                    }
                );
                assert!(match compare_context(
                    "one",
                    &to_env![("VAR1", "one"), ("VAR3", "irrelevant change")]
                ) {
                    Ok(true) if filter == &var1_and_value_eq_one => true,
                    Ok(false) if filter == &var1_and_value_ne_one => true,
                    Ok(true) if filter == &var1_and_value_eq_one_reversed => true,
                    Ok(false) if filter == &var1_and_value_ne_one_reversed => true,
                    matched => panic!("got {matched:?} for filter {filter}"),
                });
                assert!(
                    match compare_context("two", &to_env![("VAR1", "one"), ("VAR2", "two")]) {
                        Ok(false) if filter == &var1_and_value_eq_one => true,
                        Ok(true) if filter == &var1_and_value_ne_one => true,
                        Ok(false) if filter == &var1_and_value_eq_one_reversed => true,
                        Ok(true) if filter == &var1_and_value_ne_one_reversed => true,
                        matched => panic!("got {matched:?} for filter {filter}"),
                    }
                );
                assert!(
                    match compare_context("one", &to_env![("VAR1", "two"), ("VAR2", "two")]) {
                        Ok(false) if filter == &var1_and_value_eq_one => true,
                        Ok(true) if filter == &var1_and_value_ne_one => true,
                        Ok(false) if filter == &var1_and_value_eq_one_reversed => true,
                        Ok(true) if filter == &var1_and_value_ne_one_reversed => true,
                        matched => panic!("got {matched:?} for filter {filter}"),
                    }
                );
                assert!(
                    match compare_context("two", &to_env![("VAR1", "two"), ("VAR2", "two")]) {
                        Ok(false) if filter == &var1_and_value_eq_one => true,
                        Ok(true) if filter == &var1_and_value_ne_one => true,
                        Ok(false) if filter == &var1_and_value_eq_one_reversed => true,
                        Ok(true) if filter == &var1_and_value_ne_one_reversed => true,
                        matched => panic!("got {matched:?} for filter {filter}"),
                    }
                );
                assert!(match compare_context("one", &to_env![("VAR2", "two")]) {
                    Err("VAR1") if filter == &var1_and_value_eq_one => true,
                    Err("VAR1") if filter == &var1_and_value_ne_one => true,
                    Err("VAR1") if filter == &var1_and_value_eq_one_reversed => true,
                    Err("VAR1") if filter == &var1_and_value_ne_one_reversed => true,
                    matched => panic!("got {matched:?} for filter {filter}"),
                });
            }
        }

        // Any
        {
            let var1_or_value_eq_one = Filter::Any(vec![
                Context(Value(Equals(to_string!("one")))),
                Context(Env(vec![(to_string!("VAR1"), Equals(to_string!("one")))])),
            ]);
            let var1_or_value_ne_one = Filter::Not(Box::new(var1_or_value_eq_one.clone()));
            let var1_or_value_eq_one_reversed = Filter::Any(vec![
                Context(Env(vec![(to_string!("VAR1"), Equals(to_string!("one")))])),
                Context(Value(Equals(to_string!("one")))),
            ]);
            let var1_or_value_ne_one_reversed =
                Filter::Not(Box::new(var1_or_value_eq_one_reversed.clone()));
            for filter in [
                &var1_or_value_eq_one,
                &var1_or_value_ne_one,
                &var1_or_value_eq_one_reversed,
                &var1_or_value_ne_one_reversed,
            ] {
                let compare_context = CompareContext::from(filter);

                assert!(
                    match compare_context("one", &to_env![("VAR1", "one"), ("VAR2", "two")]) {
                        Ok(true) if filter == &var1_or_value_eq_one => true,
                        Ok(false) if filter == &var1_or_value_ne_one => true,
                        Ok(true) if filter == &var1_or_value_eq_one_reversed => true,
                        Ok(false) if filter == &var1_or_value_ne_one_reversed => true,
                        matched => panic!("got {matched:?} for filter {filter}"),
                    }
                );
                assert!(
                    match compare_context("two", &to_env![("VAR1", "one"), ("VAR2", "two")]) {
                        Ok(true) if filter == &var1_or_value_eq_one => true,
                        Ok(false) if filter == &var1_or_value_ne_one => true,
                        Ok(true) if filter == &var1_or_value_eq_one_reversed => true,
                        Ok(false) if filter == &var1_or_value_ne_one_reversed => true,
                        matched => panic!("got {matched:?} for filter {filter}"),
                    }
                );
                assert!(
                    match compare_context("one", &to_env![("VAR1", "two"), ("VAR2", "two")]) {
                        Ok(true) if filter == &var1_or_value_eq_one => true,
                        Ok(false) if filter == &var1_or_value_ne_one => true,
                        Ok(true) if filter == &var1_or_value_eq_one_reversed => true,
                        Ok(false) if filter == &var1_or_value_ne_one_reversed => true,
                        matched => panic!("got {matched:?} for filter {filter}"),
                    }
                );
                assert!(
                    match compare_context("two", &to_env![("VAR1", "two"), ("VAR2", "two")]) {
                        Ok(false) if filter == &var1_or_value_eq_one => true,
                        Ok(true) if filter == &var1_or_value_ne_one => true,
                        Ok(false) if filter == &var1_or_value_eq_one_reversed => true,
                        Ok(true) if filter == &var1_or_value_ne_one_reversed => true,
                        matched => panic!("got {matched:?} for filter {filter}"),
                    }
                );
                match compare_context("one", &to_env![("VAR2", "two")]) {
                    Ok(true) if filter == &var1_or_value_eq_one => true,
                    Ok(false) if filter == &var1_or_value_ne_one => true,
                    Err("VAR1") if filter == &var1_or_value_eq_one_reversed => true,
                    Err("VAR1") if filter == &var1_or_value_ne_one_reversed => true,
                    matched => panic!("got {matched:?} for filter {filter}"),
                };
                match compare_context("two", &to_env![("VAR2", "two")]) {
                    Err("VAR1") if filter == &var1_or_value_eq_one => true,
                    Err("VAR1") if filter == &var1_or_value_ne_one => true,
                    Err("VAR1") if filter == &var1_or_value_eq_one_reversed => true,
                    Err("VAR1") if filter == &var1_or_value_ne_one_reversed => true,
                    matched => panic!("got {matched:?} for filter {filter}"),
                };
            }
        }

        assert!(matches!(
            CompareContext::from(&Filter::Not(Box::new(Not(Box::new(Context(Value(
                Equals(to_string!("one")),
            )))))))("one", &HashMap::new()),
            Ok(true)
        ));
    }
}
