use anyhow::{anyhow, Error, Result};
use cxx::CxxVector;
use ms_toollib::cal_probability_onboard as ms_cal_probability_onboard;
use ms_toollib::laymine as ms_laymine;
use ms_toollib::laymine_op as ms_laymine_op;
use ms_toollib::laymine_solvable as ms_laymine_solvable;
use ms_toollib::laymine_solvable_adjust as ms_laymine_solvable_adjust;
use ms_toollib::laymine_solvable_thread as ms_laymine_solvable_thread;
use ms_toollib::AvfVideo as ms_AvfVideo;
use std::fmt;
use std::slice;
use thiserror::Error;

// cmake -B build . && make -C build -j4

#[cxx::bridge]
mod ffi {

    struct Veci32 {
        vec: Vec<i32>,
    }

    // struct CxxVeci32<'a> {
    //     cxxvec: &'a CxxVector<i32>,
    // }

    // laymine_solvable_thread的返回值类型
    struct BoardFlag {
        board: Vec<Veci32>,
        flag: bool,
    }

    struct Vecf64 {
        vec: Vec<f64>,
    }

    struct BoardPossReturn {
        board_poss: Vec<Vecf64>,
        min_mine_num: usize,
        mine_num: usize,
        max_mine_num: usize,
    }

    extern "Rust" {

        fn laymine(row: usize, column: usize, mine_num: usize, x0: usize, y0: usize)
            -> Vec<Veci32>;
        fn laymine_op(
            row: usize,
            column: usize,
            mine_num: usize,
            x0: usize,
            y0: usize,
        ) -> Vec<Veci32>;
        fn laymine_solvable_thread(
            row: usize,
            column: usize,
            mine_num: usize,
            x0: usize,
            y0: usize,
            max_times: usize,
        ) -> BoardFlag;
        fn cal_probability_onboard(
            board_of_game: &CxxVector<i32>,
            n_row: usize,
            // board_of_game: Vec<ffi::Veci32>,
            mine_num: f64,
        ) -> Result<BoardPossReturn>;
        // unsafe fn new_Veci32(ptr: *mut i32, len: usize) -> Box<CxxVeci32>;

        type AvfVideo;
        fn new_AvfVideo(file_name: &str) -> Box<AvfVideo>;
        fn parse_video(self: &mut AvfVideo);
        fn analyse(self: &mut AvfVideo);
        fn get_row(self: &AvfVideo) -> usize;
        fn get_column(self: &AvfVideo) -> usize;
        fn get_level(self: &AvfVideo) -> usize;
        fn get_win(self: &AvfVideo) -> bool;
        fn get_mine_num(self: &AvfVideo) -> usize;
        fn get_player(self: &AvfVideo) -> String;
        fn get_bbbv(self: &AvfVideo) -> usize;
        fn get_r_time(self: &AvfVideo) -> f64;
    }
}

fn laymine(row: usize, column: usize, mine_num: usize, x0: usize, y0: usize) -> Vec<ffi::Veci32> {
    let mut x = ms_laymine(row, column, mine_num, x0, y0);
    x.into_iter()
        .map(|x| ffi::Veci32 { vec: x })
        .collect::<Vec<ffi::Veci32>>()
}

fn laymine_op(
    row: usize,
    column: usize,
    mine_num: usize,
    x0: usize,
    y0: usize,
) -> Vec<ffi::Veci32> {
    let mut x = ms_laymine_op(row, column, mine_num, x0, y0);
    x.into_iter()
        .map(|x| ffi::Veci32 { vec: x })
        .collect::<Vec<ffi::Veci32>>()
}

fn laymine_solvable_thread(
    row: usize,
    column: usize,
    mine_num: usize,
    x0: usize,
    y0: usize,
    max_times: usize,
) -> ffi::BoardFlag {
    let (mut x, y) = ms_laymine_solvable_thread(row, column, mine_num, x0, y0, max_times);
    let xx = x
        .into_iter()
        .map(|x| ffi::Veci32 { vec: x })
        .collect::<Vec<ffi::Veci32>>();
    ffi::BoardFlag { board: xx, flag: y }
}

// #[derive(Error, Debug)]
// pub enum CalPossError {
//     Error200,
//     Error201,
// }

// impl fmt::Display for CalPossError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match *self {
//             CalPossError::Error200 =>
//                 write!(f, "please use a vector with at least one element"),
//             CalPossError::Error201 => write!(f, "please use a vector with at least one element"),
//         }
//     }
// }

fn cal_probability_onboard(
    board_of_game: &CxxVector<i32>,
    n_row: usize,
    // board_of_game: Vec<ffi::Veci32>,
    mine_num: f64,
) -> Result<ffi::BoardPossReturn> {
    let mut board_of_game_rust: Vec<Vec<i32>> = vec![];

    let n_column = board_of_game.len() / n_row;
    for i in 0..n_row {
        board_of_game_rust.push(vec![]);
        for j in 0..n_column {
            unsafe {
                board_of_game_rust[i].push(board_of_game.get_unchecked(i * n_column + j).clone());
            }
        }
    }
    // board_of_game_rust.iter().for_each(|x| println!("{:?}", x));
    let a = ms_cal_probability_onboard(&board_of_game_rust, mine_num);
    match a {
        Ok((b, c)) => {
            let bb = b
                .into_iter()
                .map(|x| ffi::Vecf64 { vec: x })
                .collect::<Vec<ffi::Vecf64>>();

            return Ok(ffi::BoardPossReturn {
                board_poss: bb,
                min_mine_num: c[0],
                mine_num: c[1],
                max_mine_num: c[2],
            });
        }
        Err(k) => Err(anyhow!("Error code: {}", k)),
    }
}

pub struct AvfVideo {
    core: ms_AvfVideo,
}


fn new_AvfVideo(file_name: &str) -> Box<AvfVideo> {
    let c = ms_AvfVideo::new(file_name);
    Box::new(AvfVideo { core: c })
}

impl AvfVideo {
    fn parse_video(&mut self) {
        self.core.parse_video().unwrap();
    }
    fn analyse(&mut self) {
        self.core.analyse();
    }
    fn get_row(&self) -> usize {
        self.core.height
    }
    fn get_column(&self) -> usize {
        self.core.width
    }
    fn get_level(&self) -> usize {
        self.core.level
    }
    fn get_win(&self) -> bool {
        self.core.win
    }
    fn get_mine_num(&self) -> usize {
        self.core.mine_num
    }
    fn get_player(&self) -> String {
        // 总感觉这里有问题……
        self.core.player.clone()
    }
    fn get_bbbv(&self) -> usize {
        self.core.static_params.bbbv
    }
    fn get_r_time(&self) -> f64 {
        self.core.dynamic_params.r_time
    }
}


