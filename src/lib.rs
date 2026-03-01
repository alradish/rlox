use std::ops::Not;

use wasm_bindgen::prelude::*;

pub mod parser;
pub mod scanner;

pub fn run(input: String, print_tokens: bool) {
    let tokens: Vec<scanner::Token> = scanner::scan(&input).collect();
    if print_tokens {
        println!("{}", scanner::pretty(&tokens));
    }
}

#[wasm_bindgen]
pub fn run_lox(input: String) -> String {
    // For now, just return the pretty-printed tokens
    let scanner = scanner::Scanner::scan_string(input);

    let mut output = String::new();
    if scanner.get_errors().is_empty().not() {
        scanner.get_errors().iter().for_each(|error| {
            output.push_str(&format!("{}\n", error));
        });
    }
    output.push_str(&format!("{}\n", scanner::pretty(&scanner.get_tokens())));
    output
}

#[wasm_bindgen]
pub fn tokenize(input: String) -> JsValue {
    let tokens: Vec<scanner::Token> = scanner::scan(&input).collect();
    serde_wasm_bindgen::to_value(&tokens).unwrap()
}
