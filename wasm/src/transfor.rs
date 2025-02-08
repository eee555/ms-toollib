use js_sys::Array;
use wasm_bindgen::JsValue;

pub fn vec_vec_to_js_value<T>(vec: Vec<Vec<T>>) -> JsValue
where
    JsValue: From<T>,
{
    let js_array = Array::new();
    for inner_vec in vec {
        let inner_js_array = Array::new();
        for value in inner_vec {
            inner_js_array.push(&JsValue::from(value));
        }
        js_array.push(&inner_js_array);
    }
    js_array.into()
}

pub fn js_value_to_vec_vec(vec: JsValue) -> Vec<Vec<i32>> {
    let js_array = Array::from(&vec);
    let board = js_array
        .iter()
        .map(|item| {
            let inner_array = Array::from(&item);
            inner_array
                .iter()
                .map(|e| e.as_f64().unwrap() as i32)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    board
}

