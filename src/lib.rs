use wasm_bindgen::prelude::*;

pub mod actions_toolkit;

#[wasm_bindgen]
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen()]
pub async fn run() {
    use actions_toolkit::exec::{self, ExecOptions, ExecOutput, FFIExecListeners};
    use js_sys::Map;

    let env = Map::new();
    env.set(&JsValue::from("ENV_VAR"), &JsValue::from("Value"));

    let options = ExecOptions {
        cwd: Some(env!("HOME").to_string()),
        env: Some(env),
        silent: Some(false),
        outStream: None,
        errStream: None,
        windowsVerbatimArguments: Some(false),
        failOnStdErr: Some(true),
        ignoreReturnCode: Some(false),
        delay: Some(10000),
        input: None,
        listeners: Some(FFIExecListeners {
            stdout: JsValue::undefined(),
            stderr: JsValue::undefined(),
            stdline: JsValue::undefined(),
            errline: JsValue::undefined(),
            debug: JsValue::undefined(),
        }),
    };
    log(&format!("{options:?}"));

    let tmp = exec::get_exec_output(
        "/usr/bin/bash",
        Some(vec![
            "-c".to_owned(),
            "echo \"ENV_VAR: \\\"${ENV_VAR}\\\"\"".to_owned(),
        ]),
        Some(options),
    );
    let tmp = tmp.await;
    log(&format!("{tmp:?}"));
    assert_eq!(
        tmp.map(|output| serde_wasm_bindgen::from_value::<ExecOutput>(output).unwrap()),
        Ok(ExecOutput {
            exitCode: 0,
            stdout: "ENV_VAR: \"Value\"\n".to_string(),
            stderr: "".to_string(),
        })
    );
}

#[cfg(test)]
mod tests {
    use {super::*, wasm_bindgen_test::wasm_bindgen_test};

    #[wasm_bindgen_test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
