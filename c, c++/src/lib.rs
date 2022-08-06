extern crate libc;
use ms_toollib::cal3BV as rs_cal3BV;
use ms_toollib::cal_possibility_onboard as rs_cal_possibility_onboard;
use ms_toollib::laymine as rs_laymine;
use std::alloc::{alloc, dealloc, Layout};
use std::mem;
use std::slice;

// https://avacariu.me/writing/2014/calling-rust-from-c

#[repr(C)]
pub struct Board {
    board: *mut i32,
    n_row: usize,
    n_column: usize,
}

// #[repr(C)]
// pub struct BoardPoss {
//     poss: *mut f64,
//     n_row: usize,
//     n_column: usize,
// }

#[repr(C)]
pub struct BoardPossReturn {
    board_poss: *mut f64,
    n_row: usize,
    n_column: usize,
    min_mine_num: usize,
    mine_num: usize,
    max_mine_num: usize,
}

#[no_mangle]
pub extern "C" fn cal3BV(board: *const i32, n_row: usize, n_column: usize) -> usize {
    assert!(!board.is_null());
    let array = unsafe { slice::from_raw_parts(board, n_row * n_column) };
    let mut board: Vec<Vec<i32>> = vec![];
    for i in 0..n_row {
        board.push(vec![]);
        for j in 0..n_column {
            board[i].push(array[i * n_column + j]);
        }
    }
    rs_cal3BV(&board)
}

#[no_mangle]
pub extern "C" fn laymine(
    row: usize,
    column: usize,
    MineNum: usize,
    X0: usize,
    Y0: usize,
) -> Board {
    let mut b = rs_laymine(row, column, MineNum, X0, Y0);
    let mut board = vec![];
    for mut item in b.iter_mut() {
        board.append(&mut item);
    }
    let p = board.as_mut_ptr();
    mem::forget(board);
    Board {
        board: p,
        n_row: row,
        n_column: column,
    }
}

#[no_mangle]
pub extern "C" fn cal_possibility_onboard(
    board_of_game: *const i32,
    n_row: usize,
    n_column: usize,
    mine_num: f64,
) -> BoardPossReturn {
    let mut game_board = vec![];
    let array = unsafe { slice::from_raw_parts(board_of_game, n_row * n_column) };
    for i in 0..n_row {
        game_board.push(vec![]);
        for j in 0..n_column {
            game_board[i].push(array[i * n_column + j]);
        }
    }
    let mut b = rs_cal_possibility_onboard(&game_board, mine_num).unwrap();

    let mut boardposs = vec![];
    for mut item in b.0.iter_mut() {
        boardposs.append(&mut item);
    }

    let poss = boardposs.as_mut_ptr();
    mem::forget(boardposs);
    BoardPossReturn {
        board_poss: poss,
        n_row: n_row,
        n_column: n_column,
        min_mine_num: b.1[0],
        mine_num: b.1[1],
        max_mine_num: b.1[2],
    }
}


#[no_mangle]
pub extern "C" fn free_board(board: Board) {
    // 由rust分配的局面内存，也应该由rust释放
    unsafe {
        let layout =
            Layout::from_size_align_unchecked(board.n_row * board.n_column, mem::size_of::<i32>());
        dealloc(board.board as *mut u8, layout);
    }
}

#[no_mangle]
pub extern "C" fn free_board_poss(board_poss: BoardPossReturn) {
    // 由rust分配的局面内存，也应该由rust释放
    unsafe {
        let layout =
            Layout::from_size_align_unchecked(board_poss.n_row * board_poss.n_column, mem::size_of::<i32>());
        dealloc(board_poss.board_poss as *mut u8, layout);
    }
}



