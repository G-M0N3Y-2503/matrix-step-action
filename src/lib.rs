use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use {super::*, wasm_bindgen_test::wasm_bindgen_test};

    #[wasm_bindgen_test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}