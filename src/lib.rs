use wasm_bindgen::prelude::*;

pub mod scanner;

pub fn run(input: String, print_tokens: bool) {
    let tokens: Vec<scanner::Token> = scanner::scan(&input).collect();
    if print_tokens {
        println!("{}", scanner::pretty(&tokens));
    }
}

#[wasm_bindgen]
pub fn run_lox(input: String) -> String {
    let tokens: Vec<scanner::Token> = scanner::scan(&input).collect();
    // For now, just return the pretty-printed tokens
    scanner::pretty(&tokens)
}

#[wasm_bindgen]
pub fn tokenize(input: String) -> JsValue {
    let tokens: Vec<scanner::Token> = scanner::scan(&input).collect();
    serde_wasm_bindgen::to_value(&tokens).unwrap()
}
