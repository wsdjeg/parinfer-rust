use getopts;
use std::env;
use std::fs;
use std::io;
use std::io::Read;
use std::os::unix;
use std::os::unix::io::FromRawFd;
use serde_json;
use types;
use types::*;

pub enum InputType {
    Json,
    Kakoune,
    Text
}

pub enum OutputType {
    Json,
    Kakoune,
    Text
}

pub struct Options {
    matches: getopts::Matches
}

fn options() -> getopts::Options {
    let mut options = getopts::Options::new();
    options.optflag("h", "help", "show this help message");
    options.optopt("", "input-format", "'json', 'text' (default: 'text')", "FMT");
    options.optopt("", "kakoune-previous-text-fd", "file descriptor to read Kakoune previous text from", "FD");
    options.optopt("", "kakoune-selection-fd", "file descriptor to read Kakoune buffer text from", "FD");
    options.optopt("m", "mode", "parinfer mode (indent, paren, or smart) (default: smart)", "MODE");
    options.optopt("", "output-format", "'json', 'kakoune', 'text' (default: 'text')", "FMT");
    options
}

pub fn usage() -> String {
    options().usage("Usage: parinfer-rust [options]")
}

impl Options {
    pub fn parse(args: &[String]) -> Result<Options, String> {
        options()
            .parse(args)
            .map(|m| Options {matches: m})
            .map_err(|e| e.to_string())
    }

    pub fn want_help(&self) -> bool {
        self.matches.opt_present("h")
    }

    fn mode(&self) -> &'static str {
        match self.matches.opt_str("m") {
            None => "smart",
            Some(ref s) if s == "i" || s == "indent" => "indent",
            Some(ref s) if s == "p" || s == "paren"  => "paren",
            Some(ref s) if s == "s" || s == "smart"  => "smart",
            _ => panic!("invalid mode specified for `-m`")
        }
    }

    fn input_type(&self) -> InputType {
        match self.matches.opt_str("input-format") {
            None => InputType::Text,
            Some(ref s) if s == "text" => InputType::Text,
            Some(ref s) if s == "json" => InputType::Json,
            Some(ref s) if s == "kakoune" => InputType::Kakoune,
            Some(ref s) => panic!("unknown input format `{}`", s)
        }
    }

    pub fn output_type(&self) -> OutputType {
        match self.matches.opt_str("output-format") {
            None => OutputType::Text,
            Some(ref s) if s == "text" => OutputType::Text,
            Some(ref s) if s == "json" => OutputType::Json,
            Some(ref s) if s == "kakoune" => OutputType::Kakoune,
            Some(ref s) => panic!("unknown output format `{}`", s)
        }
    }

    pub fn request(&self) -> io::Result<Request> {
        match self.input_type() {
            InputType::Text => {
                let mut text = String::new();
                io::stdin().read_to_string(&mut text)?;
                Ok(Request {
                    mode: String::from(self.mode()),
                    text,
                    options: types::Options {
                        changes: vec![],
                        cursor_x: None,
                        cursor_line: None,
                        prev_text: None,
                        prev_cursor_x: None,
                        prev_cursor_line: None,
                        force_balance: false,
                        return_parens: false,
                        partial_result: false,
                        selection_start_line: None
                    }
                })
            },
            InputType::Kakoune => {
                let selection_fd = self
                    .matches
                    .opt_str("kakoune-selection-fd")
                    .map(|s| s.parse::<unix::io::RawFd>().unwrap())
                    .expect("--kakoune-selection-fd is required");
                let mut text = String::new();
                let mut selection_file: fs::File = unsafe {FromRawFd::from_raw_fd(selection_fd)};
                selection_file.read_to_string(&mut text)?;

                let prev_text = match self.matches.opt_str("kakoune-previous-text-fd") {
                    Some(s) => {
                        let fd: unix::io::RawFd = s.parse().unwrap();
                        let mut prev_text_file: fs::File = unsafe {FromRawFd::from_raw_fd(fd)};
                        let mut prev_text = String::new();
                        prev_text_file.read_to_string(&mut prev_text)?;
                        Some(prev_text)
                    },
                    None => None
                };

                Ok  (Request {
                    mode: String::from(self.mode()),
                    text,
                    options: types::Options {
                        changes: vec![],
                        cursor_x: env::var("kak_opt_parinfer_cursor_char_column")
                            .map(|s| s.parse::<Column>().unwrap() - 1)
                            .ok(),
                        cursor_line: env::var("kak_opt_parinfer_cursor_line")
                            .map(|s| s.parse::<LineNumber>().unwrap() - 1)
                            .ok(),
                        prev_text,
                        prev_cursor_x: env::var("kak_opt_parinfer_previous_cursor_char_column")
                            .map(|s| s.parse::<Column>().unwrap() - 1)
                            .ok(),
                        prev_cursor_line: env::var("kak_opt_parinfer_previous_cursor_line")
                            .map(|s| s.parse::<LineNumber>().unwrap() - 1)
                            .ok(),
                        force_balance: false,
                        return_parens: false,
                        partial_result: false,
                        selection_start_line: None
                    }
                })
            },
            InputType::Json => {
                let mut input = String::new();
                io::stdin().read_to_string(&mut input)?;
                Ok(serde_json::from_str(&input)?)
            },
        }
    }

}
