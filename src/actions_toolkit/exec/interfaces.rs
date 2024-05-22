use super::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(no_deref)] // TODO: https://github.com/rustwasm/wasm-bindgen/issues/3964
    pub type Buffer;

    #[wasm_bindgen(no_deref, js_namespace = stream)]
    // TODO: https://github.com/rustwasm/wasm-bindgen/issues/3964
    pub type Writable;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(no_deref)] // TODO: https://github.com/rustwasm/wasm-bindgen/issues/3964
    pub type Environment;

    #[wasm_bindgen(method, indexing_getter)]
    fn get(this: &Environment, key: &str) -> JsString;

    #[wasm_bindgen(method, indexing_setter)]
    fn set(this: &Environment, key: &str, val: &str);

    #[wasm_bindgen(method, indexing_deleter)]
    fn delete(this: &Environment, key: &str);
}

/**
 * Interface for exec options
 */
#[wasm_bindgen]
pub struct ExecOptions {
    ///  optional working directory.  defaults to current
    pub cwd: Option<String>,
    ///  optional envvar dictionary.  defaults to current process's env
    pub env: Option<Environment>,
    ///  optional.  defaults to false
    pub silent: Option<bool>,
    ///  optional out stream to use. Defaults to process.stdout
    pub outStream: Option<Writable>,
    ///  optional err stream to use. Defaults to process.stderr
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
    pub input: Option<Buffer>,
    ///  optional. Listeners for output. Callback functions that will be called on these events
    pub listeners: Option<ExecListeners>,
}
/**
 * Interface for the output of getExecOutput()
 */
#[wasm_bindgen]
pub struct ExecOutput {
    /// The exit code of the process
    pub exitCode: isize,
    /// The entire stdout of the process as a string
    pub stdout: String,
    /// The entire stderr of the process as a string
    pub stderr: String,
}
/**
 * The user defined listeners for an exec call
 */
#[wasm_bindgen]
pub struct ExecListeners {
    ///  A call back for each buffer of stdout
    pub stdout: Option<&Closure<dyn FnMut(Buffer)>>,
    ///  A call back for each buffer of stderr
    pub stderr: Option<&Closure<dyn FnMut(Buffer)>>,
    ///  A call back for each line of stdout
    pub stdline: Option<&Closure<dyn FnMut(&str)>>,
    ///  A call back for each line of stderr
    pub errline: Option<&Closure<dyn FnMut(&str)>>,
    ///  A call back for each debug log
    pub debug: Option<&Closure<dyn FnMut(&str)>>,
}
