use wasm_bindgen::prelude::*;
mod util;

#[wasm_bindgen]
pub fn gen(seed: i32) -> String {
    util::gen(seed as u64).to_string()
}

#[wasm_bindgen(getter_with_clone)]
pub struct Ret {
    pub score: i64,
    pub err: String,
    pub svg: String,
}

#[wasm_bindgen]
pub fn vis(_input: String, _output: String, turn: usize) -> Ret {
    let input = util::parse_input(&_input);
    let output_result = util::parse_output(&input, &_output);
    match output_result {
        Ok(output) => {
            let (score, err, svg) = util::vis(&input, &output, turn);
            Ret {
                score: score as i64,
                err: err.to_string(),
                svg: svg.to_string(),
            }
        }
        Err(err) => Ret {
            score: 0,
            err: err.to_string(),
            svg: String::new(),
        },
    }
}

#[wasm_bindgen]
pub fn get_max_turn(_input: String, _output: String) -> usize {
    let input = util::parse_input(&_input);
    match util::parse_output(&input, &_output) {
        Ok(out) => out.out.len(),
        Err(_) => 0,
    }
}
