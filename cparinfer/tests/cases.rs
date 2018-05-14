extern crate cparinfer;
extern crate serde;
#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use std::ffi::{CStr, CString};

const INDENT_MODE_CASES: &'static str = include_str!("cases/indent-mode.json");
const PAREN_MODE_CASES: &'static str = include_str!("cases/paren-mode.json");
const SMART_MODE_CASES: &'static str = include_str!("cases/smart-mode.json");

type LineNumber = usize;
type Column = usize;

#[derive(Deserialize)]
struct Case {
    text: String,
    result: CaseResult,
    source: Source,
    options: Options,
}

impl Case {
    fn check2(&self, answer: serde_json::Value) {
        println!("answer = {}", answer);
        assert_eq!(
            json!(self.result.success), answer["success"],
            "case {}: success",
            self.source.line_no
        );
        assert_eq!(
            self.result.text, answer["text"],
            "case {}: text",
            self.source.line_no
        );

        if let Some(x) = self.result.cursor_x {
            assert_eq!(
                json!(x),
                answer["cursorX"],
                "case {}: cursor_x",
                self.source.line_no
            );
        }
        if let Some(line_no) = self.result.cursor_line {
            assert_eq!(
                json!(line_no),
                answer["cursorLine"],
                "case {}: cursor_line",
                self.source.line_no
            );
        }

        if let Some(ref expected) = self.result.error {
            assert_eq!(
                json!(expected.x), answer["error"]["x"],
                "case {}: error.x",
                self.source.line_no
            );
            assert_eq!(
                json!(expected.line_no), answer["error"]["lineNo"],
                "case {}: error.line_no",
                self.source.line_no
            );
            assert_eq!(
                json!(expected.name),
                answer["error"]["name"],
                "case {}: error.name",
                self.source.line_no
            );
        }

        if let Some(ref tab_stops) = self.result.tab_stops {
            assert_eq!(
                tab_stops.len(),
                answer["tabStops"].as_array().unwrap().len(),
                "case {}: tab stop count",
                self.source.line_no
            );
            for (expected, actual) in tab_stops.iter().zip(answer["tabStops"].as_array().unwrap().iter()) {
                assert_eq!(
                    json!(expected.ch), actual["ch"],
                    "case {}: tab stop ch",
                    self.source.line_no
                );
                assert_eq!(
                    json!(expected.x), actual["x"],
                    "case {}: tab stop x",
                    self.source.line_no
                );
                assert_eq!(
                    json!(expected.line_no), actual["lineNo"],
                    "case {}: tab stop line",
                    self.source.line_no
                );
                assert_eq!(
                    json!(expected.arg_x), actual["argX"],
                    "case {}: tab stop arg_x",
                    self.source.line_no
                );
            }
        }

        if let Some(ref trails) = self.result.paren_trails {
            assert_eq!(
                trails.len(),
                answer["parenTrails"].as_array().unwrap().len(),
                "case {}: wrong number of paren trails",
                self.source.line_no
            );
            for (expected, actual) in trails.iter().zip(answer["parenTrails"].as_array().unwrap().iter()) {
                assert_eq!(
                    expected.line_no, actual["lineNo"],
                    "case {}: paren trail line number",
                    self.source.line_no
                );
                assert_eq!(
                    expected.start_x, actual["startX"],
                    "case {}: paren trail start x",
                    self.source.line_no
                );
                assert_eq!(
                    expected.end_x, actual["endX"],
                    "case {}: paren trail end x",
                    self.source.line_no
                );
            }
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Options {
    cursor_x: Option<Column>,
    cursor_line: Option<LineNumber>,
    #[serde(skip_serializing_if = "Option::is_none")]
    changes: Option<Vec<Change>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prev_cursor_x: Option<Column>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prev_cursor_line: Option<LineNumber>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Change {
    line_no: LineNumber,
    x: Column,
    old_text: String,
    new_text: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TabStop {
    ch: String,
    x: Column,
    line_no: LineNumber,
    arg_x: Option<Column>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CaseResult {
    text: String,
    success: bool,
    error: Option<Error>,
    cursor_x: Option<Column>,
    cursor_line: Option<LineNumber>,
    tab_stops: Option<Vec<TabStop>>,
    paren_trails: Option<Vec<ParenTrail>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ParenTrail {
    line_no: LineNumber,
    start_x: Column,
    end_x: Column,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Error {
    name: String,
    line_no: LineNumber,
    x: Column,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Source {
    line_no: LineNumber,
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
pub fn indent_mode() {
    let cases: Vec<Case> = serde_json::from_str(INDENT_MODE_CASES).unwrap();
    for case in cases {
        let input = CString::new(json!({
            "mode": "indent",
            "text": &case.text,
            "options": &case.options
        }).to_string()).unwrap();
        let answer: serde_json::Value = unsafe {
            let out = CStr::from_ptr(cparinfer::run_parinfer(input.as_ptr())).to_str().unwrap();
            serde_json::from_str(out).unwrap()
        };
        case.check2(answer);
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
pub fn paren_mode() {
    let cases: Vec<Case> = serde_json::from_str(PAREN_MODE_CASES).unwrap();
    for case in cases {
        let input = CString::new(json!({
            "mode": "paren",
            "text": &case.text,
            "options": &case.options
        }).to_string()).unwrap();
        let answer: serde_json::Value = unsafe {
            let out = CStr::from_ptr(cparinfer::run_parinfer(input.as_ptr())).to_str().unwrap();
            serde_json::from_str(out).unwrap()
        };
        case.check2(answer);
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
pub fn smart_mode() {
    let cases: Vec<Case> = serde_json::from_str(SMART_MODE_CASES).unwrap();
    for case in cases {
        let input = CString::new(json!({
            "mode": "smart",
            "text": &case.text,
            "options": &case.options
        }).to_string()).unwrap();
        let answer: serde_json::Value = unsafe {
            let out = CStr::from_ptr(cparinfer::run_parinfer(input.as_ptr())).to_str().unwrap();
            serde_json::from_str(out).unwrap()
        };
        case.check2(answer);
    }
}