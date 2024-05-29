#![cfg(test)]

use matrix_step_action::actions_toolkit;
use {wasm_bindgen::prelude::*, wasm_bindgen_test::wasm_bindgen_test};

#[wasm_bindgen_test]
async fn test_core_bindings() {
    use actions_toolkit::core::{
        self,
        summary::{SummaryImageOptions, SummaryTable, SummaryTableCell, SummaryWriteOptions},
        AnnotationProperties,
    };
    use js_sys::{Error, Promise};

    core::export_variable("name", "val".into());
    core::set_secret("secret");
    core::add_path("inputPath");
    core::get_input(
        "name",
        Some(core::InputOptions {
            required: Some(true),
            trim_whitespace: Some(true),
        }),
    )
    .unwrap_err();
    core::get_multiline_input(
        "name",
        Some(core::InputOptions {
            required: Some(true),
            trim_whitespace: Some(true),
        }),
    )
    .unwrap_err();
    core::get_boolean_input(
        "name",
        Some(core::InputOptions {
            required: Some(true),
            trim_whitespace: Some(true),
        }),
    )
    .unwrap_err();
    core::set_output("name", "value".into());
    core::set_command_echo(true);
    assert!(!core::is_debug());
    core::debug("message");
    core::error(
        Error::new("message").into(),
        Some(AnnotationProperties {
            title: Some("title".to_owned()),
            file: Some(file!().to_owned()),
            start_line: Some(1),
            end_line: None,
            start_column: Some(1),
            end_column: Some(2),
        }),
    );
    core::warning(
        Error::new("message").into(),
        Some(AnnotationProperties {
            title: Some("title".to_owned()),
            file: Some(file!().to_owned()),
            start_line: Some(1),
            end_line: None,
            start_column: Some(1),
            end_column: Some(2),
        }),
    );
    core::notice(
        "message".into(),
        Some(AnnotationProperties {
            title: Some("title".to_owned()),
            file: Some(file!().to_owned()),
            start_line: Some(1),
            end_line: None,
            start_column: Some(1),
            end_column: Some(2),
        }),
    );
    core::info("message");
    core::start_group("name");
    core::end_group();
    assert_eq!(
        core::group("name", &mut || Promise::new(&mut |resolve, _reject| {
            resolve.call1(&JsValue::undefined(), &true.into()).unwrap();
        }))
        .await
        .unwrap(),
        JsValue::from(true)
    );
    assert_eq!(
        core::group("name", &mut || Promise::new(&mut |_resolve, reject| {
            reject.call1(&JsValue::undefined(), &true.into()).unwrap();
        }))
        .await
        .unwrap_err(),
        JsValue::from(true)
    );
    core::save_state("name", "value".into());
    assert_eq!(core::get_state("name"), "");
    core::get_id_token(Some("aud")).await.unwrap_err();

    core::SUMMARY
        .write(Some(SummaryWriteOptions {
            overwrite: Some(true),
        }))
        .await
        .unwrap_err();
    core::SUMMARY.clear().await.unwrap_err();
    assert_eq!(core::SUMMARY.stringify(), "");
    assert!(core::SUMMARY.is_empty_buffer());
    core::SUMMARY.add_raw("text", Some(true));
    core::SUMMARY.empty_buffer();
    core::SUMMARY.add_eol();
    core::SUMMARY.add_code_block("code", Some("lang"));
    core::SUMMARY.add_list(
        vec!["1".to_owned(), "2".to_owned(), "3".to_owned()],
        Some(true),
    );
    core::SUMMARY.add_table(
        serde_wasm_bindgen::to_value(&SummaryTable(vec![
            vec![SummaryTableCell {
                data: "Title".to_owned(),
                header: Some(true),
                colspan: Some(2),
                rowspan: Some(2),
            }],
            vec![
                SummaryTableCell {
                    data: "data1".to_owned(),
                    header: None,
                    colspan: None,
                    rowspan: None,
                },
                SummaryTableCell {
                    data: "data2".to_owned(),
                    header: None,
                    colspan: None,
                    rowspan: None,
                },
            ],
        ]))
        .unwrap(),
    );
    core::SUMMARY.add_details("label", "content");
    core::SUMMARY.add_image(
        "src",
        "alt",
        Some(SummaryImageOptions {
            width: Some(500),
            height: Some(500),
        }),
    );
    core::SUMMARY.add_heading("text", Some(4));
    core::SUMMARY.add_separator();
    core::SUMMARY.add_break();
    core::SUMMARY.add_quote("text", Some("cite"));
    core::SUMMARY.add_link("text", "href");

    assert_eq!(
        String::from(core::to_platform_path("pth")),
        "pth".to_owned()
    );
    assert_eq!(String::from(core::to_posix_path("pth")), "pth".to_owned());
    assert_eq!(String::from(core::to_win32_path("pth")), "pth".to_owned());
}

#[wasm_bindgen_test]
#[ignore = "Causes processes to exit with 1"]
fn test_core_set_failed_bindings() {
    use actions_toolkit::core;
    use js_sys::Error;

    core::set_failed(Error::new("message"));
}

#[wasm_bindgen_test]
async fn test_exec_bindings() {
    use actions_toolkit::exec::{self, ExecOptions, FFIExecListeners};
    use js_sys::Map;

    let env = Map::new();
    env.set(&JsValue::from("ENV_VAR"), &JsValue::from("Value"));
    let tmp = exec::exec(
        "echo",
        Some(vec!["\"${ENV_VAR}\"".to_owned()]),
        Some(ExecOptions {
            cwd: Some(env!("CARGO_TARGET_TMPDIR").to_string()),
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
        }),
    );
    println!("test!");
    assert_eq!(tmp.await.unwrap().as_f64(), Some(0f64));
}
