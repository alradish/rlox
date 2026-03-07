use std::ops::Not;

use wasm_bindgen::prelude::*;

pub mod parser;
pub mod scanner;

#[wasm_bindgen]
pub fn parse_to_ast(input: String) -> JsValue {
    let tokens = scanner::scan(&input).collect();
    let mut parser = parser::Parser::new(tokens);
    match parser.parse() {
        Ok(ast) => serde_wasm_bindgen::to_value(&ast).unwrap(),
        Err(e) => JsValue::from_str(&format!("Parser error: {}", e)),
    }
}

pub fn run(input: String, print_tokens: bool) {
    let tokens = scanner::scan(&input).collect();
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
