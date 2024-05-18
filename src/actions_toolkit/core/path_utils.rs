use super::*;

#[wasm_bindgen(module = "@actions/core")]
extern "C" {
    /// toPosixPath converts the given path to the posix form. On Windows, \\ will be
    /// replaced with /.
    ///
    /// @param pth. Path to transform.
    /// @return string Posix path.
    ///
    #[wasm_bindgen(js_name = "toPosixPath")]
    pub fn to_posix_path(pth: &str) -> JsString;
    /// toWin32Path converts the given path to the win32 form. On Linux, / will be
    /// replaced with \\.
    ///
    /// @param pth. Path to transform.
    /// @return string Win32 path.
    ///
    #[wasm_bindgen(js_name = "toWin32Path")]
    pub fn to_win32_path(pth: &str) -> JsString;
    /// toPlatformPath converts the given path to a platform-specific path. It does
    /// this by replacing instances of / and \ with the platform-specific path
    /// separator.
    ///
    /// @param pth The path to platformize.
    /// @return string The platform-specific path.
    #[wasm_bindgen(js_name = "toPlatformPath")]
    pub fn to_platform_path(pth: &str) -> JsString;
}
