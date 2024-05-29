use {super::*, js_sys::JsString, std::collections::HashMap};

#[wasm_bindgen]
extern "C" {
    /// Node JS Buffer type
    #[derive(Deserialize, Clone, PartialEq, Debug, Default)]
    #[wasm_bindgen(no_deref)] // TODO: https://github.com/rustwasm/wasm-bindgen/issues/3964
    pub type Buffer;

    /// Node JS stream.Writable type
    #[derive(Deserialize, Clone, PartialEq, Debug, Default)]
    #[wasm_bindgen(no_deref, js_namespace = stream)]
    // TODO: https://github.com/rustwasm/wasm-bindgen/issues/3964
    pub type Writable;
}

/// Interface for exec options
#[wasm_bindgen]
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct ExecOptions {
    ///  optional working directory.  defaults to current
    #[wasm_bindgen(getter_with_clone)]
    pub cwd: Option<String>,
    ///  optional envvar dictionary.  defaults to current process's env
    #[wasm_bindgen(getter_with_clone)]
    pub env: Option<HashMap<String, String>>,
    ///  optional.  defaults to false
    pub silent: Option<bool>,
    ///  optional out stream to use. Defaults to process.stdout
    #[wasm_bindgen(getter_with_clone)]
    pub outStream: Option<Writable>,
    ///  optional err stream to use. Defaults to process.stderr
    #[wasm_bindgen(getter_with_clone)]
    pub errStream: Option<Writable>,
    ///  optional. whether to skip quoting/escaping arguments if needed.  defaults to false.
    pub windowsVerbatimArguments: Option<bool>,
    ///  optional.  whether to fail if output to stderr.  defaults to false
    pub failOnStdErr: Option<bool>,
    ///  optional.  defaults to failing on non zero.  ignore will not fail leaving it up to the caller
    pub ignoreReturnCode: Option<bool>,
    ///  optional. How long in ms to wait for STDIO streams to close after the exit event of the process before terminating. defaults to 10000
    pub delay: Option<usize>,
    ///  optional. input to write to the process on STDIN.
    #[wasm_bindgen(getter_with_clone)]
    pub input: Option<Buffer>,
    ///  optional. Listeners for output. Callback functions that will be called on these events
    #[wasm_bindgen(getter_with_clone)]
    pub listeners: Option<FFIExecListeners>,
}
/// Interface for the output of getExecOutput()
#[wasm_bindgen]
#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct ExecOutput {
    /// The exit code of the process
    pub exitCode: isize,
    /// The entire stdout of the process as a string
    #[wasm_bindgen(getter_with_clone)]
    pub stdout: String,
    /// The entire stderr of the process as a string
    #[wasm_bindgen(getter_with_clone)]
    pub stderr: String,
}
/// The user defined listeners for an exec call
// #[derive(Serialize, Deserialize)]
#[derive(Clone, PartialEq, Debug, Default)]
#[wasm_bindgen(getter_with_clone)]
pub struct FFIExecListeners {
    ///  A call back for each buffer of stdout
    pub stdout: JsValue,
    ///  A call back for each buffer of stderr
    pub stderr: JsValue,
    ///  A call back for each line of stdout
    pub stdline: JsValue,
    ///  A call back for each line of stderr
    pub errline: JsValue,
    ///  A call back for each debug log
    pub debug: JsValue,
}
#[wasm_bindgen]
pub struct ExecListeners {
    ///  A call back for each buffer of stdout
    pub stdout: &'static dyn FnMut(Buffer) -> String,
    ///  A call back for each buffer of stderr
    pub stderr: &'static dyn FnMut(Buffer) -> String,
    ///  A call back for each line of stdout
    pub stdline: &'static dyn FnMut(JsString) -> String,
    ///  A call back for each line of stderr
    pub errline: &'static dyn FnMut(JsString) -> String,
    ///  A call back for each debug log
    pub debug: &'static dyn FnMut(JsString) -> String,
}

// impl ExecListeners<'_> {
//     pub fn to_ffi(&self) -> ExecListeners {}
// }
