use ms_toollib_original as ms;

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

// use serde::{Deserialize, Serialize};
use serde_json;

use wasm_bindgen::prelude::*;
// use web_sys;
mod board;
// use board::{MinesweeperBoard,AvfVideo};
mod transfor;
// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

pub fn set_panic_hook() {
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn cal_bbbv(board_json: &str) -> i32 {
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
    ms::cal_bbbv(&res) as i32
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
    ms::cal_op(&res) as i32
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
    let _ = ms::mark_board(&mut board_of_game);
    match ms::cal_possibility_onboard(&board_of_game, mine_num as f64) {
        Ok(t) => return serde_json::to_string(&t).unwrap(),
        Err(_) => return serde_json::to_string(&(Vec::<i32>::new(), [0, 0, 0])).unwrap(),
    }
}

#[wasm_bindgen]
pub fn laymine(row: i32, column: i32, mine_num: i32, x0: i32, y0: i32) -> String {
    serde_json::to_string(&ms::laymine(
        row as usize,
        column as usize,
        mine_num as usize,
        x0 as usize,
        y0 as usize,
    ))
    .unwrap()
}

#[wasm_bindgen]
pub fn laymine_op(row: i32, column: i32, mine_num: i32, x0: i32, y0: i32) -> String {
    serde_json::to_string(&ms::laymine_op(
        row as usize,
        column as usize,
        mine_num as usize,
        x0 as usize,
        y0 as usize,
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
    max_times: i32,
) -> String {
    serde_json::to_string(&ms::laymine_solvable(
        row as usize,
        column as usize,
        mine_num as usize,
        x0 as usize,
        y0 as usize,
        max_times as usize,
    ))
    .unwrap()
}

#[wasm_bindgen]
pub fn is_solvable(board_json: &str, x0: i32, y0: i32) -> bool {
    let board_: serde_json::Value = serde_json::from_str(&board_json).unwrap();
    let board__ = board_.as_array().unwrap();
    let len_ = board__.len();
    let mut board: Vec<_> = vec![];
    for i in 0..len_ {
        board.push(
            board__[i]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_i64().unwrap() as i32)
                .collect::<Vec<_>>(),
        );
    }
    ms::is_solvable(&board, x0 as usize, y0 as usize)
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
