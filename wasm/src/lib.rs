use ms;

// cargo install wasm-pack
// cargo install wasm-bindgen-cli
// cargo generate --git https://github.com/rustwasm/wasm-pack-template
// npm init wasm-app www
// cd www
// npm install
// npm start
// 教程：https://www.secondstate.io/articles/getting-started-with-rust-function/
// npm config set registry https://registry.npm.taobao.org
// npm config set registry https://registry.npmjs.org

// 打包给webpack等bundler使用：wasm-pack build
// wasm-pack build --debug
// 打包给nodejs使用：wasm-pack build --target nodejs
// 发布wasm-pack publish
// 发布wasm-pack publish --target nodejs
// npm config set registry https://registry.npmmirror.com/
// npm config set registry https://registry.npm.taobao.org/
// npm config set registry https://registry.npmjs.org/

use js_sys::Array;
use wasm_bindgen::prelude::*;
mod board;
// use board::{MinesweeperBoard,AvfVideo};
mod transfor;
use transfor::{js_value_to_vec_vec, vec_vec_to_js_value};

#[wasm_bindgen]
pub fn cal_bbbv(js_board: JsValue) -> usize {
    let board = js_value_to_vec_vec(js_board);
    ms::cal_bbbv(&board)
}

#[wasm_bindgen]
pub fn cal_op(js_board: JsValue) -> usize {
    let board = js_value_to_vec_vec(js_board);
    ms::cal_op(&board)
}

#[wasm_bindgen]
pub fn cal_possibility_onboard(js_board: JsValue, mine_num: f64) -> JsValue {
    let mut game_board = js_value_to_vec_vec(js_board);
    let _ = ms::mark_board(&mut game_board);
    let array = js_sys::Array::new();
    match ms::cal_possibility_onboard(&game_board, mine_num) {
        Ok(t) => {
            array.push(&vec_vec_to_js_value(t.0));
            let mine_num_array = Array::new_with_length(3);
            mine_num_array.set(0, JsValue::from(t.1[0]));
            mine_num_array.set(1, JsValue::from(t.1[1]));
            mine_num_array.set(2, JsValue::from(t.1[2]));
            array.push(&mine_num_array);
            return array.into();
        }
        Err(t) => JsValue::from(t),
    }
}

#[wasm_bindgen]
pub fn laymine(row: usize, column: usize, mine_num: usize, x0: usize, y0: usize) -> JsValue {
    let board = ms::laymine(row, column, mine_num, x0, y0);
    vec_vec_to_js_value(board)
}

#[wasm_bindgen]
pub fn laymine_op(row: usize, column: usize, mine_num: usize, x0: usize, y0: usize) -> JsValue {
    let board = ms::laymine_op(row, column, mine_num, x0, y0);
    vec_vec_to_js_value(board)
}

#[wasm_bindgen]
pub fn laymine_solvable(
    row: usize,
    column: usize,
    mine_num: usize,
    x0: usize,
    y0: usize,
    max_times: usize,
) -> JsValue {
    let board_flag = ms::laymine_solvable(row, column, mine_num, x0, y0, max_times);
    let array = js_sys::Array::new();
    array.push(&vec_vec_to_js_value(board_flag.0));
    array.push(&JsValue::from_bool(board_flag.1));
    array.into()
}

#[wasm_bindgen]
pub fn is_solvable(js_board: JsValue, x0: usize, y0: usize) -> bool {
    let board = js_value_to_vec_vec(js_board);
    ms::is_solvable(&board, x0, y0)
}

#[wasm_bindgen(getter_with_clone)]
pub struct TimePeriod {
    pub start_time: String,
    pub end_time: String,
}

#[wasm_bindgen]
pub fn valid_time_period(software: &str) -> TimePeriod {
    let (start_time, end_time) = ms::valid_time_period(software).unwrap();
    TimePeriod {
        start_time,
        end_time,
    }
}
