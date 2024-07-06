use wasm_bindgen::prelude::*;

mod matrix;
pub use matrix::EnvVars;

mod input;

#[macro_export]
macro_rules! prop_builder {
    ($obj:block$(.$setter:ident($value:expr))*) => {{
        let tmp = $obj;
        $(tmp.$setter($value);)*
        tmp
    }};
}

#[wasm_bindgen(module = "node:process")]
extern "C" {
    pub type Env;
    #[wasm_bindgen(js_name = "env")]
    pub static ENV: Env;

    #[wasm_bindgen(method, structural, indexing_getter)]
    pub fn get(this: &Env, variable: &str) -> js_sys::JsString;

    #[wasm_bindgen(method, structural, indexing_setter)]
    pub fn set(this: &Env, variable: &str, value: &str);

    #[wasm_bindgen(method, structural, indexing_deleter)]
    pub fn delete(this: &Env, variable: &str);
}

#[wasm_bindgen()]
pub async fn run() {
    use actions::{
        core::{debug, error, info},
        exec::{self, Env, ExecListeners, ExecOptions, ExecOutput},
    };

    let print_stdout = Closure::new(|str: js_sys::JsString| {
        info(&format!("{str:?}"));
    });
    let print_stderr = Closure::new(|str: js_sys::JsString| {
        error(format!("Error: {str:?}").into(), None);
    });
    let print_debug = Closure::new(|str: js_sys::JsString| {
        debug(&format!("Debug: {str:?}"));
    });

    ENV.set("test", "value");

    let env = Env::default();
    env.set("ENV_VAR", "Value");
    assert_eq!(
        exec::get_exec_output(
            "/usr/bin/bash",
            Some(vec![
                "--noprofile".to_owned(),
                "--norc".to_owned(),
                "-c".to_owned(),
                "echo \"ENV_VAR: \\\"${ENV_VAR}\\\"\"".to_owned(),
            ]),
            Some(prop_builder! {
                { ExecOptions::default() }
                .set_cwd(Some("/".to_string()))
                .set_env(Some(env))
                .set_silent(Some(false))
                .set_out_stream(None)
                .set_err_stream(None)
                .set_windows_verbatim_arguments(Some(false))
                .set_fail_on_std_err(Some(true))
                .set_ignore_return_code(Some(false))
                .set_delay(Some(10000))
                .set_input(None)
                .set_listeners(Some(prop_builder! {
                    { ExecListeners::default() }
                        .set_stdline(&print_stdout)
                        .set_errline(&print_stderr)
                        .set_debug(&print_debug)
                }))
            }),
        )
        .await
        .map(Into::into),
        Ok(prop_builder! {
            { ExecOutput::default() }
                .set_exit_code(0)
                .set_stdout("ENV_VAR: \"Value\"\n".to_string())
                .set_stderr("".to_string())
        })
    );
}

#[cfg(test)]
mod tests {
    use {super::*, wasm_bindgen_test::wasm_bindgen_test};

    #[ignore = "Runs main without dependencies"]
    #[wasm_bindgen_test]
    async fn can_run() {
        run().await;
    }
}
