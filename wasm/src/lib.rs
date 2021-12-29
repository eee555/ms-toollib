use ms_toollib as ms_toollib_js;

// cargo generate --git https://github.com/rustwasm/wasm-pack-template
// wasm-pack build
// npm init wasm-app www
// cd www
// npm install
// npm start
// 教程：https://www.secondstate.io/articles/getting-started-with-rust-function/
// npm config set registry https://registry.npm.taobao.org
// npm config set registry https://registry.npmjs.org

// 打包给webpack等bundler使用：wasm-pack build
// 打包给nodejs使用：wasm-pack build --target nodejs
// 发布wasm-pack publish


use serde::{Deserialize, Serialize};
use serde_json;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("888");
}

#[wasm_bindgen]
pub fn cal3BV(board_json: &str) -> i32 {
    // set_panic_hook();
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
    ms_toollib_js::cal3BV(&res) as i32
}

#[wasm_bindgen]
pub fn cal_op(board_json: &str) -> i32 {
    // set_panic_hook();
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
    ms_toollib_js::cal_op(res) as i32
}

#[wasm_bindgen]
pub fn cal_possibility_onboard(board_json: &str, mine_num: i32) -> String {
    // set_panic_hook();
    let board_: serde_json::Value = serde_json::from_str(&board_json).unwrap();
    let board__ = board_.as_array().unwrap();
    let len_ = board__.len();
    let mut board_of_game = vec![];
    for i in 0..len_ {
        board_of_game.push(
            board__[i]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_i64().unwrap() as i32)
                .collect::<Vec<_>>(),
        );
    }
    // mine_num为局面中雷的总数，不管有没有标
    ms_toollib_js::mark_board(&mut board_of_game);
    match ms_toollib_js::cal_possibility_onboard(&board_of_game, mine_num as f64) {
        Ok(t) => return serde_json::to_string(&t).unwrap(),
        Err(_) => return serde_json::to_string(&(Vec::<i32>::new(), [0, 0, 0])).unwrap(),
    };
}

#[wasm_bindgen]
pub fn laymine_number(row: i32, column: i32, mine_num: i32, x0: i32, y0: i32) -> String {
    serde_json::to_string(&ms_toollib_js::laymine_number(
        row as usize,
        column as usize,
        mine_num as usize,
        x0 as usize,
        y0 as usize,
    ))
    .unwrap()
}

#[wasm_bindgen]
pub fn laymine_op_number(row: i32, column: i32, mine_num: i32, x0: i32, y0: i32) -> String {
    serde_json::to_string(&ms_toollib_js::laymine_op_number(
        row as usize,
        column as usize,
        mine_num as usize,
        x0 as usize,
        y0 as usize,
    ))
    .unwrap()
}

#[wasm_bindgen]
pub fn laymine(
    row: i32,
    column: i32,
    mine_num: i32,
    x0: i32,
    y0: i32,
    min_3BV: i32,
    max_3BV: i32,
    max_times: i32,
    method: i32,
) -> String {
    serde_json::to_string(&ms_toollib_js::laymine(
        row as usize,
        column as usize,
        mine_num as usize,
        x0 as usize,
        y0 as usize,
        min_3BV as usize,
        max_3BV as usize,
        max_times as usize,
        method as usize,
    ))
    .unwrap()
}

#[wasm_bindgen]
pub fn laymine_op(
    row: i32,
    column: i32,
    mine_num: i32,
    x0: i32,
    y0: i32,
    min_3BV: i32,
    max_3BV: i32,
    max_times: i32,
    method: i32,
) -> String {
    serde_json::to_string(&ms_toollib_js::laymine_op(
        row as usize,
        column as usize,
        mine_num as usize,
        x0 as usize,
        y0 as usize,
        min_3BV as usize,
        max_3BV as usize,
        max_times as usize,
        method as usize,
    ))
    .unwrap()
}

#[wasm_bindgen]
pub fn laymine_solvable(
    row: i32,
    column: i32,
    mine_num: i32,
    x0: i32,
    y0: i32,
    min_3BV: i32,
    max_3BV: i32,
    max_times: i32,
    enu_limit: i32,
) -> String {
    serde_json::to_string(&ms_toollib_js::laymine_solvable(
        row as usize,
        column as usize,
        mine_num as usize,
        x0 as usize,
        y0 as usize,
        min_3BV as usize,
        max_3BV as usize,
        max_times as usize,
        enu_limit as usize,
    ))
    .unwrap()
}

