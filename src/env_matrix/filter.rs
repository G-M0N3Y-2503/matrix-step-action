mod retry_fold;

use {
    super::*,
    core::{
        cell::{RefCell, RefMut},
        fmt::Display,
        ops::{ControlFlow, Not},
    },
    log::*,
    retry_fold::retry_fold,
    std::{
        collections::{hash_map, HashMap},
        rc::Rc,
    },
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
pub type CompareContext<'comparer> =
    Box<dyn FnMut() -> Result<bool, hash_map::VacantEntry<'comparer, String, String>> + 'comparer>;
impl Filter {
    pub fn to_predicate<'comparer>(
        &'comparer self,
        value: &'comparer str,
        env: &'comparer mut HashMap<String, String>,
    ) -> CompareContext<'comparer> {
        type Error<'comparer> = hash_map::VacantEntry<'comparer, String, String>;
        type Result<'comparer> = core::result::Result<bool, Error<'comparer>>;

        #[inline]
        fn retry_fold_filters<'c>(
            filters: impl IntoIterator<Item = &'c Filter> + 'c,
            init: bool,
            fold: impl FnMut(
                    &mut bool,
                    CompareContext<'c>,
                ) -> ControlFlow<(Result<'c>, CompareContext<'c>)>
                + 'c,
            value: &'c str,
            env: &'c mut HashMap<String, String>,
        ) -> CompareContext<'c> {
            let mut filters = itertools::put_back(
                filters
                    .into_iter()
                    .map(|filter| filter.to_predicate(value, env)),
            );
            let mut retry_fold = retry_fold(init, fold);
            Box::new(move || match retry_fold(&mut filters) {
                ControlFlow::Continue(res) | ControlFlow::Break(Ok(res)) => Ok(res),
                ControlFlow::Break(Err(err)) => Err(err),
            })
        }

        #[inline]
        fn try_all<'c>(
            _initial: &mut bool,
            mut comparer: CompareContext<'c>,
        ) -> ControlFlow<(Result<'c>, CompareContext<'c>)> {
            const CONTINUE_VALUE: bool = true;
            debug_assert_eq!(*_initial, CONTINUE_VALUE);
            match comparer() {
                Ok(CONTINUE_VALUE) => ControlFlow::Continue(()),
                res @ Ok(false) | res @ Err(_) => ControlFlow::Break((res, comparer)),
            }
        }

        #[inline]
        fn try_any<'c>(
            _initial: &mut bool,
            mut comparer: CompareContext<'c>,
        ) -> ControlFlow<(Result<'c>, CompareContext<'c>)> {
            const CONTINUE_VALUE: bool = false;
            debug_assert_eq!(*_initial, CONTINUE_VALUE);
            match comparer() {
                Ok(CONTINUE_VALUE) => ControlFlow::Continue(()),
                res @ Ok(true) | res @ Err(_) => ControlFlow::Break((res, comparer)),
            }
        }

        match self {
            Filter::All(filters) => retry_fold_filters(filters, true, try_all, value, env),
            Filter::Any(filters) => retry_fold_filters(filters, false, try_any, value, env),
            parrent_filter @ Filter::Not(filter) => {
                let filter = filter.as_ref();
                if let Filter::Not(_) = filter {
                    warn!("Double negative filter found at:\n{parrent_filter}");
                }
                let mut compare = filter.to_predicate(value, env);
                Box::new(move || compare().map(Not::not))
            }
            Filter::Context(FilterContext::Value(comparer)) => {
                let comparer = CompareValue::from(comparer);
                Box::new(move || Ok(comparer(value)))
            }
            Filter::Context(FilterContext::Env(comparers)) => {
                let env = Rc::new(env);
                let mut comparers = itertools::put_back(comparers.iter().map(
                    move |(env_var, comparer): &(String, _)| -> CompareContext {
                        let env = RefCell::new(env);
                        let comparer = CompareValue::from(comparer);
                        Box::new(move || {
                            match RefMut::map(env.borrow_mut(), |env| {
                                &mut env.entry(env_var.to_string())
                            }) {
                                hash_map::Entry::Occupied(occupied) => Ok(comparer(occupied.get())),
                                hash_map::Entry::Vacant(vacant) => Err(vacant),
                            }
                        })
                    },
                ));
                let mut retry_fold = retry_fold(true, try_all);
                Box::new(move || match retry_fold(&mut comparers) {
                    ControlFlow::Continue(res) | ControlFlow::Break(Ok(res)) => Ok(res),
                    ControlFlow::Break(Err(err)) => Err(err),
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

impl Display for FilterContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
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
type CompareValue<'comparer> = Box<dyn Fn(&str) -> bool + 'comparer>;
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
        [$(($variable:expr, $value:expr)),*] => {
            HashMap::<String, String, std::hash::RandomState>::from_iter([$((to_string!($variable), to_string!($value))),*])
        };
    }

    #[collapse_debuginfo(no)]
    macro_rules! test_filter_predicate {
        (
            filters!{$(let $filter_name:ident = $filter_expr:expr;)+};
            $(for_each_filter!{
                predicate_with!($value:expr, $env:expr);
                $(return_matches!($filter_match:ident, $pattern:pat);)+
            };)+
        ) => {
            $(let $filter_name = $filter_expr;)+
            let test_reuse = 0..4;
            for filter in [$(&$filter_name),+] {
                $(
                    let env = $env;
                    let mut predicate = filter.to_predicate($value, &env);
                    for try_index in test_reuse.clone() {
                        assert!(match predicate() {
                           $($pattern if filter == &$filter_match => true,)+
                            matched => panic!(
                                "Unexpected `return_matches!({filter_name}, {matched:?})` for `predicate_with!({value}, {env})` on run {try_num}",
                                filter_name = match filter {
                                    $(_ if filter == &$filter_match => stringify!($filter_match).to_owned(),)+
                                    filter => format!("{filter:?}"),
                                },
                                value = stringify!($value),
                                env = stringify!($env),
                                try_num = try_index + 1,
                            ),
                        });
                    }
                )+
            }
        };
    }

    #[test]
    #[wasm_bindgen_test]
    fn value_filter_test() {
        test_filter_predicate! {
            filters!{
                let value_eq_one = Filter::Context(Value(Equals(to_string!("one"))));
                let value_ne_one = Filter::Not(Box::new(value_eq_one.clone()));
            };
            for_each_filter!{
                predicate_with!("one", &to_env![("VAR1", "one"), ("VAR2", "two")]);
                return_matches!(value_eq_one, Ok(true));
                return_matches!(value_ne_one, Ok(false));
            };
            for_each_filter!{
                predicate_with!("one", &to_env![("VAR3", "irrelevant change")]);
                return_matches!(value_eq_one, Ok(true));
                return_matches!(value_ne_one, Ok(false));
            };
            for_each_filter!{
                predicate_with!("two", &to_env![("VAR1", "one"), ("VAR2", "two")]);
                return_matches!(value_eq_one, Ok(false));
                return_matches!(value_ne_one, Ok(true));
            };
            for_each_filter!{
                predicate_with!("two", &to_env![("VAR3", "irrelevant change")]);
                return_matches!(value_eq_one, Ok(false));
                return_matches!(value_ne_one, Ok(true));
            };
        };
    }

    #[test]
    #[wasm_bindgen_test]
    fn env_filter_test() {
        test_filter_predicate! {
            filters!{
                let var1_eq_one = Filter::Context(Env(vec![(to_string!("VAR1"), Equals(to_string!("one")))]));
                let var1_ne_one = Filter::Not(Box::new(var1_eq_one.clone()));
                let var1_var3 = Filter::Context(Env(vec![
                    (to_string!("VAR1"), Equals(to_string!("one"))),
                    (to_string!("VAR3"), Equals(to_string!("three"))),
                ]));
                let var3_var1 = Filter::Context(Env(vec![
                    (to_string!("VAR3"), Equals(to_string!("three"))),
                    (to_string!("VAR1"), Equals(to_string!("one"))),
                ]));
            };
            for_each_filter!{
                predicate_with!("one", &to_env![("VAR1", "one"), ("VAR2", "two")]);
                return_matches!(var1_eq_one, Ok(true));
                return_matches!(var1_ne_one, Ok(false));
                return_matches!(var1_var3, Err("VAR3"));
                return_matches!(var3_var1, Err("VAR3"));
            };
            for_each_filter!{
                predicate_with!("irrelevant change", &to_env![("VAR1", "one"), ("VAR2", "two")]);
                return_matches!(var1_eq_one, Ok(true));
                return_matches!(var1_ne_one, Ok(false));
                return_matches!(var1_var3, Err("VAR3"));
                return_matches!(var3_var1, Err("VAR3"));
            };
            for_each_filter!{
                predicate_with!("one", &to_env![("VAR1", "one"), ("VAR3", "irrelevant change")]);
                return_matches!(var1_eq_one, Ok(true));
                return_matches!(var1_ne_one, Ok(false));
                return_matches!(var1_var3, Ok(false));
                return_matches!(var3_var1, Ok(false));
            };
            for_each_filter!{
                predicate_with!("one", &to_env![("VAR1", "two"), ("VAR2", "two")]);
                return_matches!(var1_eq_one, Ok(false));
                return_matches!(var1_ne_one, Ok(true));
                return_matches!(var1_var3, Ok(false));
                return_matches!(var3_var1, Err("VAR3"));
            };
            for_each_filter!{
                predicate_with!("one", &to_env![("VAR2", "two")]);
                return_matches!(var1_eq_one, Err("VAR1"));
                return_matches!(var1_ne_one, Err("VAR1"));
                return_matches!(var1_var3, Err("VAR1"));
                return_matches!(var3_var1, Err("VAR3"));
            };
            for_each_filter!{
                predicate_with!("one", &to_env![("VAR1", "one"), ("VAR2", "two"), ("VAR3", "three")]);
                return_matches!(var1_eq_one, Ok(true));
                return_matches!(var1_ne_one, Ok(false));
                return_matches!(var1_var3, Ok(true));
                return_matches!(var3_var1, Ok(true));
            };
            for_each_filter!{
                predicate_with!("one", &to_env![("VAR3", "three"), ("VAR2", "two"), ("VAR1", "one")]);
                return_matches!(var1_eq_one, Ok(true));
                return_matches!(var1_ne_one, Ok(false));
                return_matches!(var1_var3, Ok(true));
                return_matches!(var3_var1, Ok(true));
            };
            for_each_filter!{
                predicate_with!("one", &to_env![("VAR1", "one"), ("VAR2", "irrelevant change"), ("VAR3", "three")]);
                return_matches!(var1_eq_one, Ok(true));
                return_matches!(var1_ne_one, Ok(false));
                return_matches!(var1_var3, Ok(true));
                return_matches!(var3_var1, Ok(true));
            };
            for_each_filter!{
                predicate_with!("one", &to_env![("VAR1", "one"), ("VAR3", "three")]);
                return_matches!(var1_eq_one, Ok(true));
                return_matches!(var1_ne_one, Ok(false));
                return_matches!(var1_var3, Ok(true));
                return_matches!(var3_var1, Ok(true));
            };
            for_each_filter!{
                predicate_with!("one", &to_env![("VAR3", "three")]);
                return_matches!(var1_eq_one, Err("VAR1"));
                return_matches!(var1_ne_one, Err("VAR1"));
                return_matches!(var1_var3, Err("VAR1"));
                return_matches!(var3_var1, Err("VAR1"));
            };
            for_each_filter!{
                predicate_with!("one", &to_env![("VAR1", "one")]);
                return_matches!(var1_eq_one, Ok(true));
                return_matches!(var1_ne_one, Ok(false));
                return_matches!(var1_var3, Err("VAR3"));
                return_matches!(var3_var1, Err("VAR3"));
            };
            for_each_filter!{
                predicate_with!("one", &HashMap::new());
                return_matches!(var1_eq_one, Err("VAR1"));
                return_matches!(var1_ne_one, Err("VAR1"));
                return_matches!(var1_var3, Err("VAR1"));
                return_matches!(var3_var1, Err("VAR3"));
            };
            for_each_filter!{
                predicate_with!("one", &to_env![("VAR1", "two"), ("VAR2", "two"), ("VAR3", "three")]);
                return_matches!(var1_eq_one, Ok(false));
                return_matches!(var1_ne_one, Ok(true));
                return_matches!(var1_var3, Ok(false));
                return_matches!(var3_var1, Ok(false));
            };
            for_each_filter!{
                predicate_with!("one", &to_env![("VAR3", "three"), ("VAR2", "two"), ("VAR1", "two")]);
                return_matches!(var1_eq_one, Ok(false));
                return_matches!(var1_ne_one, Ok(true));
                return_matches!(var1_var3, Ok(false));
                return_matches!(var3_var1, Ok(false));
            };
            for_each_filter!{
                predicate_with!("one", &to_env![("VAR1", "one"), ("VAR2", "two"), ("VAR3", "two")]);
                return_matches!(var1_eq_one, Ok(true));
                return_matches!(var1_ne_one, Ok(false));
                return_matches!(var1_var3, Ok(false));
                return_matches!(var3_var1, Ok(false));
            };
            for_each_filter!{
                predicate_with!("one", &to_env![("VAR3", "two"), ("VAR2", "two"), ("VAR1", "one")]);
                return_matches!(var1_eq_one, Ok(true));
                return_matches!(var1_ne_one, Ok(false));
                return_matches!(var1_var3, Ok(false));
                return_matches!(var3_var1, Ok(false));
            };
            for_each_filter!{
                predicate_with!("one", &to_env![("VAR1", "two"), ("VAR2", "two"), ("VAR3", "two")]);
                return_matches!(var1_eq_one, Ok(false));
                return_matches!(var1_ne_one, Ok(true));
                return_matches!(var1_var3, Ok(false));
                return_matches!(var3_var1, Ok(false));
            };
            for_each_filter!{
                predicate_with!("one", &to_env![("VAR3", "two"), ("VAR2", "two"), ("VAR1", "two")]);
                return_matches!(var1_eq_one, Ok(false));
                return_matches!(var1_ne_one, Ok(true));
                return_matches!(var1_var3, Ok(false));
                return_matches!(var3_var1, Ok(false));
            };
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn all_filter_test() {
        test_filter_predicate! {
            filters!{
                let var1_and_value_eq_one = Filter::All(vec![
                    Context(Env(vec![(to_string!("VAR1"), Equals(to_string!("one")))])),
                    Context(Value(Equals(to_string!("one")))),
                ]);
                let var1_and_value_ne_one = Filter::Not(Box::new(var1_and_value_eq_one.clone()));
                let var1_and_value_eq_one_reversed = Filter::All(vec![
                    Context(Value(Equals(to_string!("one")))),
                    Context(Env(vec![(to_string!("VAR1"), Equals(to_string!("one")))])),
                ]);
                let var1_and_value_ne_one_reversed = Filter::Not(Box::new(var1_and_value_eq_one_reversed.clone()));
            };
            for_each_filter!{
                predicate_with!("one", &to_env![("VAR1", "one"), ("VAR2", "two")]);
                return_matches!(var1_and_value_eq_one, Ok(true));
                return_matches!(var1_and_value_ne_one, Ok(false));
                return_matches!(var1_and_value_eq_one_reversed, Ok(true));
                return_matches!(var1_and_value_ne_one_reversed, Ok(false));
            };
            for_each_filter!{
                predicate_with!("one", &to_env![("VAR1", "one"), ("VAR3", "irrelevant change")]);
                return_matches!(var1_and_value_eq_one, Ok(true));
                return_matches!(var1_and_value_ne_one, Ok(false));
                return_matches!(var1_and_value_eq_one_reversed, Ok(true));
                return_matches!(var1_and_value_ne_one_reversed, Ok(false));
            };
            for_each_filter!{
                predicate_with!("two", &to_env![("VAR1", "one"), ("VAR2", "two")]);
                return_matches!(var1_and_value_eq_one, Ok(false));
                return_matches!(var1_and_value_ne_one, Ok(true));
                return_matches!(var1_and_value_eq_one_reversed, Ok(false));
                return_matches!(var1_and_value_ne_one_reversed, Ok(true));
            };
            for_each_filter!{
                predicate_with!("one", &to_env![("VAR1", "two"), ("VAR2", "two")]);
                return_matches!(var1_and_value_eq_one, Ok(false));
                return_matches!(var1_and_value_ne_one, Ok(true));
                return_matches!(var1_and_value_eq_one_reversed, Ok(false));
                return_matches!(var1_and_value_ne_one_reversed, Ok(true));
            };
            for_each_filter!{
                predicate_with!("two", &to_env![("VAR1", "two"), ("VAR2", "two")]);
                return_matches!(var1_and_value_eq_one, Ok(false));
                return_matches!(var1_and_value_ne_one, Ok(true));
                return_matches!(var1_and_value_eq_one_reversed, Ok(false));
                return_matches!(var1_and_value_ne_one_reversed, Ok(true));
            };
            for_each_filter!{
                predicate_with!("one", &to_env![("VAR2", "two")]);
                return_matches!(var1_and_value_eq_one, Err("VAR1"));
                return_matches!(var1_and_value_ne_one, Err("VAR1"));
                return_matches!(var1_and_value_eq_one_reversed, Err("VAR1"));
                return_matches!(var1_and_value_ne_one_reversed, Err("VAR1"));
            };
        };
    }

    #[test]
    #[wasm_bindgen_test]
    fn any_filter_test() {
        test_filter_predicate! {
            filters!{
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
            };
            for_each_filter!{
                predicate_with!("one", &to_env![("VAR1", "one"), ("VAR2", "two")]);
                return_matches!(var1_or_value_eq_one, Ok(true));
                return_matches!(var1_or_value_ne_one, Ok(false));
                return_matches!(var1_or_value_eq_one_reversed, Ok(true));
                return_matches!(var1_or_value_ne_one_reversed, Ok(false));
            };
            for_each_filter!{
                predicate_with!("two", &to_env![("VAR1", "one"), ("VAR2", "two")]);
                return_matches!(var1_or_value_eq_one, Ok(true));
                return_matches!(var1_or_value_ne_one, Ok(false));
                return_matches!(var1_or_value_eq_one_reversed, Ok(true));
                return_matches!(var1_or_value_ne_one_reversed, Ok(false));
            };
            for_each_filter!{
                predicate_with!("one", &to_env![("VAR1", "two"), ("VAR2", "two")]);
                return_matches!(var1_or_value_eq_one, Ok(true));
                return_matches!(var1_or_value_ne_one, Ok(false));
                return_matches!(var1_or_value_eq_one_reversed, Ok(true));
                return_matches!(var1_or_value_ne_one_reversed, Ok(false));
            };
            for_each_filter!{
                predicate_with!("two", &to_env![("VAR1", "two"), ("VAR2", "two")]);
                return_matches!(var1_or_value_eq_one, Ok(false));
                return_matches!(var1_or_value_ne_one, Ok(true));
                return_matches!(var1_or_value_eq_one_reversed, Ok(false));
                return_matches!(var1_or_value_ne_one_reversed, Ok(true));
            };
            for_each_filter!{
                predicate_with!("one", &to_env![("VAR2", "two")]);
                return_matches!(var1_or_value_eq_one, Ok(true));
                return_matches!(var1_or_value_ne_one, Ok(false));
                return_matches!(var1_or_value_eq_one_reversed, Err("VAR1"));
                return_matches!(var1_or_value_ne_one_reversed, Err("VAR1"));
            };
            for_each_filter!{
                predicate_with!("two", &to_env![("VAR2", "two")]);
                return_matches!(var1_or_value_eq_one, Err("VAR1"));
                return_matches!(var1_or_value_ne_one, Err("VAR1"));
                return_matches!(var1_or_value_eq_one_reversed, Err("VAR1"));
                return_matches!(var1_or_value_ne_one_reversed, Err("VAR1"));
            };
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn not_filter_test() {
        test_filter_predicate! {
            filters!{
                let not_not = Filter::Not(Box::new(Not(Box::new(Context(Value(Equals(to_string!(
                    "one"
                ))))))));
            };
            for_each_filter!{
                predicate_with!("one", &to_env![]);
                return_matches!(not_not, Ok(true));
            };
        }
    }
}
