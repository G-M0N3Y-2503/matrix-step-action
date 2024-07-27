use {super::*, actions::core::InputOptions};

const ENV_VARS_INPUT_KEY: &str = "ENV_VARS";

pub async fn parse_input() -> Result<EnvMatrix, js_sys::Error> {
    match actions::core::get_input(
        ENV_VARS_INPUT_KEY,
        Some(prop_builder!(
            { InputOptions::default() }.set_required(Some(true))
        )),
    ) {
        Ok(json) => json
            .as_string()
            .ok_or(js_sys::Error::new("Input is not a string")),
        Err(err) => Err(js_sys::Error::from(err)),
    }
    .and_then(|json_str| {
        serde_json::from_str(&json_str)
            .map_err(|err| js_sys::Error::from(JsValue::from(JsError::from(err))))
    })
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        serde_json::json,
        wasm_bindgen_test::{console_log, wasm_bindgen_test},
    };

    #[wasm_bindgen(module = "node:process")]
    extern "C" {
        #[wasm_bindgen(js_name = "env")]
        pub static ENV: js_sys::Object;
    }

    pub fn set_github_input(var: &str, val: &str) {
        console_log!("{var}={val}");
        js_sys::Reflect::set(
            &ENV,
            &JsValue::from(format!("INPUT_{var}")),
            &JsValue::from(val),
        )
        .unwrap();
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_input_parse() {
        console_log!(
            "{}",
            serde_json::to_string(&EnvMatrix::from_iter([(
                "VAR1".to_owned(),
                env_matrix::ValuesFrom::Combinations {
                    of: 2,
                    from: Box::new(env_matrix::ValuesFrom::Values(vec![
                        "1".to_owned(),
                        "2".to_owned(),
                        "3".to_owned(),
                        "4".to_owned()
                    ])),
                    join_with: ",".to_owned()
                },
            )]))
            .unwrap()
        );
        // {
        //     let input: String = json!({"VAR1": {"EachOf": ["Value1"]}}).to_string();
        //     let expected: EnvVars = HashMap::from([(
        //         "VAR1".to_owned(),
        //         ValueFrom::EachOf(vec!["Value1".to_owned()]),
        //     )]);
        //     set_github_input(ENV_VARS_INPUT_KEY, &input);

        //     assert_eq!(parse_input().await, Ok(expected));
        // }
    }
}
