use serde::Serialize;
use serde_json;
// use wasm_bindgen::prelude::*;
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

// 从json的二维矩阵转为rust里的二维vector
pub fn json2vec(board_json: &str) -> Vec<Vec<i32>> {
    let board_: serde_json::Value = serde_json::from_str(&board_json).unwrap();
    let board__ = board_.as_array().unwrap();
    let len_ = board__.len();
    let mut res = vec![];
    for i in 0..len_ {
        res.push(
            board__[i]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_i64().unwrap() as i32)
                .collect::<Vec<_>>(),
        );
    }
    res
}

// 从rust里的二维vector转为json的二维矩阵
pub fn vec2json<T: Serialize>(board_json: &Vec<Vec<T>>) -> String {
    serde_json::to_string(&board_json).unwrap()
}

pub fn trans_opt(operation: &str) -> Vec<(String, (usize, usize))> {
    let board_: serde_json::Value = serde_json::from_str(operation).unwrap();
    let board__ = board_.as_array().unwrap();
    let len_ = board__.len();
    let mut res = vec![];
    for i in 0..len_ {
        res.push((
            board__[i].get(0).unwrap().as_str().unwrap().to_string(),
            (
                board__[i].get(1).unwrap().as_u64().unwrap() as usize,
                board__[i].get(2).unwrap().as_u64().unwrap() as usize,
            ),
        ));
    }
    res
}
