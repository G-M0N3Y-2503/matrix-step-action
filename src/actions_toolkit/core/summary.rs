use super::*;

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct SummaryTableCell {
    /// Cell content
    #[wasm_bindgen(getter_with_clone)]
    pub data: String,
    /// Render cell as header
    /// (optional) default: false
    pub header: Option<bool>,
    /// Number of columns the cell extends
    /// (optional) default: '1'
    #[wasm_bindgen(getter_with_clone)]
    pub colspan: Option<usize>,
    /// Number of rows the cell extends
    /// (optional) default: '1'
    #[wasm_bindgen(getter_with_clone)]
    pub rowspan: Option<usize>,
}

#[wasm_bindgen]
#[derive(
    Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default,
)]
pub struct SummaryImageOptions {
    /// The width of the image in pixels. Must be an integer without a unit.
    /// (optional)
    #[wasm_bindgen(getter_with_clone)]
    pub width: Option<usize>,
    /// The height of the image in pixels. Must be an integer without a unit.
    /// (optional)
    #[wasm_bindgen(getter_with_clone)]
    pub height: Option<usize>,
}
#[wasm_bindgen]
#[derive(
    Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default,
)]
pub struct SummaryWriteOptions {
    /// Replace all existing content in summary file with buffer contents
    /// (optional) default: false
    pub overwrite: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct SummaryTable(pub Vec<Vec<SummaryTableCell>>);

#[wasm_bindgen(module = "@actions/core")]
extern "C" {
    #[allow(clippy::empty_docs)]
    #[wasm_bindgen(no_deref)] // TODO: https://github.com/rustwasm/wasm-bindgen/issues/3964
    #[derive(Clone, PartialEq, Debug, Default)]
    pub type Summary;

    #[allow(clippy::empty_docs)]
    #[wasm_bindgen(js_name = "summary")]
    pub static SUMMARY: Summary;
    /// Writes text in the buffer to the summary buffer file and empties buffer. Will append by default.
    ///
    /// @param {SummaryWriteOptions} [options] (optional) options for write operation
    ///
    /// @returns {Promise<Summary>} summary instance
    #[wasm_bindgen(method, catch)]
    pub async fn write(
        this: &Summary,
        options: Option<SummaryWriteOptions>,
    ) -> Result<JsValue, JsValue>;
    /// Clears the summary buffer and wipes the summary file
    ///
    /// @returns {Summary} summary instance
    #[wasm_bindgen(method, catch)]
    pub async fn clear(this: &Summary) -> Result<JsValue, JsValue>;
    /// Returns the current summary buffer as a string
    ///
    /// @returns {string} string of summary buffer
    #[wasm_bindgen(method)]
    pub fn stringify(this: &Summary) -> JsString;
    /// If the summary buffer is empty
    ///
    /// @returns {boolean} true if the buffer is empty
    #[wasm_bindgen(method, js_name = "isEmptyBuffer")]
    pub fn is_empty_buffer(this: &Summary) -> bool;
    /// Resets the summary buffer without writing to summary file
    ///
    /// @returns {Summary} summary instance
    #[wasm_bindgen(method, js_name = "emptyBuffer")]
    pub fn empty_buffer(this: &Summary) -> Summary;
    /// Adds raw text to the summary buffer
    ///
    /// @param {string} text content to add
    /// @param {boolean} [addEOL=false] (optional) append an EOL to the raw text (default: false)
    ///
    /// @returns {Summary} summary instance
    #[wasm_bindgen(method, js_name = "addRaw")]
    pub fn add_raw(this: &Summary, text: &str, add_eol: Option<bool>) -> Summary;
    /// Adds the operating system-specific end-of-line marker to the buffer
    ///
    /// @returns {Summary} summary instance
    #[wasm_bindgen(method, js_name = "addEOL")]
    pub fn add_eol(this: &Summary) -> Summary;
    /// Adds an HTML codeblock to the summary buffer
    ///
    /// @param {string} code content to render within fenced code block
    /// @param {string} lang (optional) language to syntax highlight code
    ///
    /// @returns {Summary} summary instance
    #[wasm_bindgen(method, js_name = "addCodeBlock")]
    pub fn add_code_block(this: &Summary, code: &str, lang: Option<&str>) -> Summary;
    /// Adds an HTML list to the summary buffer
    ///
    /// @param {string[]} items list of items to render
    /// @param {boolean} [ordered=false] (optional) if the rendered list should be ordered or not (default: false)
    ///
    /// @returns {Summary} summary instance
    #[wasm_bindgen(method, js_name = "addList")]
    pub fn add_list(this: &Summary, items: Vec<String>, ordered: Option<bool>) -> Summary;
    /// Adds an HTML table to the summary buffer
    ///
    /// @param {SummaryTableCell[]} rows table rows
    ///
    /// @returns {Summary} summary instance
    #[wasm_bindgen(method, js_name = "addTable")]
    pub fn add_table(this: &Summary, summary_table: JsValue) -> Summary;
    /// Adds a collapsable HTML details element to the summary buffer
    ///
    /// @param {string} label text for the closed state
    /// @param {string} content collapsable content
    ///
    /// @returns {Summary} summary instance
    #[wasm_bindgen(method, js_name = "addDetails")]
    pub fn add_details(this: &Summary, label: &str, content: &str) -> Summary;
    /// Adds an HTML image tag to the summary buffer
    ///
    /// @param {string} src path to the image you to embed
    /// @param {string} alt text description of the image
    /// @param {SummaryImageOptions} options (optional) addition image attributes
    ///
    /// @returns {Summary} summary instance
    #[wasm_bindgen(method, js_name = "addImage")]
    pub fn add_image(
        this: &Summary,
        src: &str,
        alt: &str,
        options: Option<SummaryImageOptions>,
    ) -> Summary;
    /// Adds an HTML section heading element
    ///
    /// @param {string} text heading text
    /// @param {number | string} [level=1] (optional) the heading level, default: 1
    ///
    /// @returns {Summary} summary instance
    #[wasm_bindgen(method, js_name = "addHeading")]
    pub fn add_heading(this: &Summary, text: &str, level: Option<usize>) -> Summary;
    /// Adds an HTML thematic break (<hr>) to the summary buffer
    ///
    /// @returns {Summary} summary instance
    #[wasm_bindgen(method, js_name = "addSeparator")]
    pub fn add_separator(this: &Summary) -> Summary;
    /// Adds an HTML line break (<br>) to the summary buffer
    ///
    /// @returns {Summary} summary instance
    #[wasm_bindgen(method, js_name = "addBreak")]
    pub fn add_break(this: &Summary) -> Summary;
    /// Adds an HTML blockquote to the summary buffer
    ///
    /// @param {string} text quote text
    /// @param {string} cite (optional) citation url
    ///
    /// @returns {Summary} summary instance
    #[wasm_bindgen(method, js_name = "addQuote")]
    pub fn add_quote(this: &Summary, text: &str, cite: Option<&str>) -> Summary;
    /// Adds an HTML anchor tag to the summary buffer
    ///
    /// @param {string} text link text/content
    /// @param {string} href hyperlink
    ///
    /// @returns {Summary} summary instance
    #[wasm_bindgen(method, js_name = "addLink")]
    pub fn add_link(this: &Summary, text: &str, href: &str) -> Summary;
}
