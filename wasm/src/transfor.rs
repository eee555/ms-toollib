use serde::Serialize;
use serde_json;
// use wasm_bindgen::prelude::*;

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



