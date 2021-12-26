mod utils;
pub use utils::{
    cal3BV, calOp, cal_table_minenum_recursion, combine, enuOneStep, layMineOpNumber,
    lay_mine_number, refreshBoard, refresh_matrix, refresh_matrixs, set_panic_hook,
    unsolvableStructure,
};
mod algorithms;
pub use algorithms::{
    cal_is_op_possibility_cells, cal_possibility, cal_possibility_onboard, isSolvable, layMine,
    layMineOp, layMineSolvable, layMineSolvable_thread, mark_board, SolveDirect, SolveEnumerate,
    SolveMinus,
};
// cargo generate --git https://github.com/rustwasm/wasm-pack-template
// wasm-pack build
// npm init wasm-app www
// npm install
// npm start
// 教程：https://www.secondstate.io/articles/getting-started-with-rust-function/
// npm config set registry https://registry.npm.taobao.org
// npm config set registry https://registry.npmjs.org


use serde::{Serialize, Deserialize};
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
pub fn wasm_cal3BV(board_json: &str) -> i32 {
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
    cal3BV(&res) as i32
}

#[wasm_bindgen]
pub fn wasm_cal_possibility_onboard(board_json: &str, mine_num: i32) -> String {
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
    mark_board(&mut board_of_game);
    match cal_possibility_onboard(&board_of_game, mine_num as f64) {
        Ok(t) => return serde_json::to_string(&t).unwrap(),
        Err(e) => return serde_json::to_string(&(Vec::<i32>::new(), [0, 0, 0])).unwrap(),
    };
}
