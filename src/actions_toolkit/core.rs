use {
    super::*,
    js_sys::{Error, JsString, Promise},
};

/// Interface for getInput options
#[wasm_bindgen]
pub struct InputOptions {
    /// Optional. Whether the input is required. If required and not present, will throw. Defaults to false
    pub required: Option<bool>,
    /// Optional. Whether leading/trailing whitespace will be trimmed for the input. Defaults to true
    #[wasm_bindgen(js_name = "trimWhitespace")]
    pub trim_whitespace: Option<bool>,
}
/// The code to exit an action*
#[wasm_bindgen]
pub enum ExitCode {
    /// A code indicating that the action was successful
    Success = 0,
    /// A code indicating that the action was a failure
    Failure = 1,
}
/// Optional properties that can be sent with annotation commands (notice, error, and warning)
/// See: https://docs.github.com/en/rest/reference/checks#create-a-check-run for more information about annotations.
#[wasm_bindgen]
pub struct AnnotationProperties {
    /// A title for the annotation.
    #[wasm_bindgen(getter_with_clone)]
    pub title: Option<String>,
    /// The path of the file for which the annotation should be created.
    #[wasm_bindgen(getter_with_clone)]
    pub file: Option<String>,
    /// The start line for the annotation.
    #[wasm_bindgen(js_name = "startLine")]
    pub start_line: Option<usize>,
    /// The end line for the annotation. Defaults to `startLine` when `startLine` is provided.
    #[wasm_bindgen(js_name = "endLine")]
    pub end_line: Option<usize>,
    /// The start column for the annotation. Cannot be sent when `startLine` and `endLine` are different values.
    #[wasm_bindgen(js_name = "startColumn")]
    pub start_column: Option<usize>,
    /// The end column for the annotation. Cannot be sent when `startLine` and `endLine` are different values.
    /// Defaults to `startColumn` when `startColumn` is provided.
    #[wasm_bindgen(js_name = "endColumn")]
    pub end_column: Option<usize>,
}

#[wasm_bindgen(module = "@actions/core")]
extern "C" {
    /// Sets env variable for this action and future actions in the job
    /// @param name the name of the variable to set
    /// @param val the value of the variable. Non-string values will be converted to a string via JSON.stringify
    #[wasm_bindgen(js_name = "exportVariable")]
    pub fn export_variable(name: &str, val: JsValue);
    /// Registers a secret which will get masked from logs
    /// @param secret value of the secret
    #[wasm_bindgen(js_name = "setSecret")]
    pub fn set_secret(secret: &str);
    /// Prepends inputPath to the PATH (for this action and future actions)
    /// @param inputPath
    #[wasm_bindgen(js_name = "addPath")]
    pub fn add_path(input_path: &str);
    /// Gets the value of an input.
    /// Unless trimWhitespace is set to false in InputOptions, the value is also trimmed.
    /// Returns an empty string if the value is not defined.
    ///
    /// @param     name     name of the input to get
    /// @param     options  optional. See InputOptions.
    /// @returns   string
    #[wasm_bindgen(catch, js_name = "getInput")]
    pub fn get_input(name: &str, options: Option<InputOptions>) -> Result<JsString, JsValue>;
    /// Gets the values of an multiline input.  Each value is also trimmed.
    ///
    /// @param     name     name of the input to get
    /// @param     options  optional. See InputOptions.
    /// @returns   string[]
    ///
    #[wasm_bindgen(catch, js_name = "getMultilineInput")]
    pub fn get_multiline_input(
        name: &str,
        options: Option<InputOptions>,
    ) -> Result<Vec<JsString>, JsValue>;
    /// Gets the input value of the boolean type in the YAML 1.2 "core schema" specification.
    /// Support boolean input list: `true | True | TRUE | false | False | FALSE` .
    /// The return value is also in boolean type.
    /// ref: https://yaml.org/spec/1.2/spec.html#id2804923
    ///
    /// @param     name     name of the input to get
    /// @param     options  optional. See InputOptions.
    /// @returns   boolean
    #[wasm_bindgen(catch, js_name = "getBooleanInput")]
    pub fn get_boolean_input(name: &str, options: Option<InputOptions>) -> Result<bool, JsValue>;
    /// Sets the value of an output.
    ///
    /// @param     name     name of the output to set
    /// @param     value    value to store. Non-string values will be converted to a string via JSON.stringify
    #[wasm_bindgen(js_name = "setOutput")]
    pub fn set_output(name: &str, value: JsValue);
    /// Enables or disables the echoing of commands into stdout for the rest of the step.
    /// Echoing is disabled by default if ACTIONS_STEP_DEBUG is not set.
    ///
    #[wasm_bindgen(js_name = "setCommandEcho")]
    pub fn set_command_echo(enabled: bool);
    /// Sets the action status to failed.
    /// When the action exits it will be with an exit code of 1
    /// @param message add error issue message
    #[wasm_bindgen(js_name = "setFailed")]
    pub fn set_failed(message: Error);
    /// Gets whether Actions Step Debug is on or not
    #[wasm_bindgen(js_name = "isDebug")]
    pub fn is_debug() -> bool;
    /// Writes debug message to user log
    /// @param message debug message
    pub fn debug(message: &str);
    /// Adds an error issue
    /// @param message error issue message. Errors will be converted to string via toString()
    /// @param properties optional properties to add to the annotation.
    pub fn error(message: JsValue, properties: Option<AnnotationProperties>);
    /// Adds a warning issue
    /// @param message warning issue message. Errors will be converted to string via toString()
    /// @param properties optional properties to add to the annotation.
    pub fn warning(message: JsValue, properties: Option<AnnotationProperties>);
    /// Adds a notice issue
    /// @param message notice issue message. Errors will be converted to string via toString()
    /// @param properties optional properties to add to the annotation.
    pub fn notice(message: JsValue, properties: Option<AnnotationProperties>);
    /// Writes info to log with console.log.
    /// @param message info message
    pub fn info(message: &str);
    /// Begin an output group.
    ///
    /// Output until the next `groupEnd` will be foldable in this group
    ///
    /// @param name The name of the output group
    #[wasm_bindgen(js_name = "startGroup")]
    pub fn start_group(name: &str);
    /// End an output group.
    #[wasm_bindgen(js_name = "endGroup")]
    pub fn end_group();
    /// Wrap an asynchronous function call in a group.
    ///
    /// Returns the same type as the function itself.
    ///
    /// @param name The name of the group
    /// @param fn The function to wrap in the group
    #[wasm_bindgen(catch)]
    pub async fn group(name: &str, r#fn: &mut dyn FnMut() -> Promise) -> Result<JsValue, JsValue>;
    /// Saves state for current action, the state can only be retrieved by this action's post job execution.
    ///
    /// @param     name     name of the state to store
    /// @param     value    value to store. Non-string values will be converted to a string via JSON.stringify
    #[wasm_bindgen(js_name = "saveState")]
    pub fn save_state(name: &str, value: JsValue);
    /// Gets the value of an state set by this action's main execution.
    ///
    /// @param     name     name of the state to get
    /// @returns   string
    #[wasm_bindgen(js_name = "getState")]
    pub fn get_state(name: &str) -> JsString;
    /// You can use these methods to interact with the GitHub OIDC provider and get a JWT ID token which would help to get access token from third party cloud providers.
    /// @returns Result<string, JsValue>
    #[wasm_bindgen(catch, js_name = "getIDToken")]
    pub async fn get_id_token(audience: Option<&str>) -> Result<JsValue, JsValue>;
}

/// Summary exports
pub mod summary;
pub use summary::SUMMARY;
/// Path exports
pub mod path_utils;
pub use path_utils::{to_platform_path, to_posix_path, to_win32_path};
