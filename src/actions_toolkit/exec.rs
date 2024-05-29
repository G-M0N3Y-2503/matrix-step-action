use super::*;

pub mod interfaces;
pub use interfaces::{ExecOptions, ExecOutput, FFIExecListeners};

#[wasm_bindgen(module = "@actions/exec")]
extern "C" {
    ///  Exec a command.
    ///  Output will be streamed to the live console.
    ///  Returns promise with return code
    ///
    ///  @param     commandLine        command to execute (can include additional args). Must be correctly escaped.
    ///  @param     args               optional arguments for tool. Escaping is handled by the lib.
    ///  @param     options            optional exec options.  See ExecOptions
    ///  @returns   Promise<number>    exit code
    #[wasm_bindgen(catch)]
    pub async fn exec(
        commandLine: &str,
        args: Option<Vec<String>>,
        options: Option<ExecOptions>,
    ) -> Result<JsValue, JsValue>;
    ///  Exec a command and get the output.
    ///  Output will be streamed to the live console.
    ///  Returns promise with the exit code and collected stdout and stderr
    ///
    ///  @param     commandLine           command to execute (can include additional args). Must be correctly escaped.
    ///  @param     args                  optional arguments for tool. Escaping is handled by the lib.
    ///  @param     options               optional exec options.  See ExecOptions
    ///  @returns   Promise<ExecOutput>   exit code, stdout, and stderr
    #[wasm_bindgen(catch, js_name = "getExecOutput")]
    pub async fn get_exec_output(
        commandLine: &str,
        args: Option<Vec<String>>,
        options: Option<ExecOptions>,
    ) -> Result<JsValue, JsValue>;
}
