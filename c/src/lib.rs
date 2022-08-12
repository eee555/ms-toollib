extern crate libc;
use ms_toollib::cal3BV as rs_cal3BV;
use ms_toollib::cal_possibility_onboard as rs_cal_possibility_onboard;
use ms_toollib::laymine as rs_laymine;
use ms_toollib::MinesweeperBoard as RustMinesweeperBoard;
use std::alloc::{alloc, dealloc, Layout};
use std::mem;
use std::ptr;
use std::slice;
use libc::c_char;
use std::ffi::CStr;

// https://avacariu.me/writing/2014/calling-rust-from-c

// #[repr(C)]
// pub struct Board {
//     board: *mut i32,
//     n_row: usize,
//     n_column: usize,
// }

#[repr(C)]
pub struct Row {
    cells: *mut i32,
    n_column: usize,
}

#[repr(C)]
pub struct Board {
    rows: *mut Row,
    n_row: usize,
}

#[repr(C)]
pub struct RowPoss {
    cells_poss: *mut f64,
    n_column: usize,
}

#[repr(C)]
pub struct BoardPoss {
    rows_poss: *mut RowPoss,
    n_row: usize,
}

#[repr(C)]
pub struct BoardPossReturn {
    board_poss: BoardPoss,
    min_mine_num: usize,
    mine_num: usize,
    max_mine_num: usize,
}

pub struct Pointer {
    x: usize,
    y: usize,
}

#[repr(C)]
pub struct MinesweeperBoard {
    board: Board,
    game_board: Board,
    flagedList: *mut Pointer,
    left: usize,
    right: usize,
    chording: usize,
    ces: usize,
    flag: usize,
    solved3BV: usize,
    row: usize,
    column: usize,
    mouse_state: MouseState,
    game_board_state: GameBoardState,
    pointer_x: usize,
    pointer_y: usize,
}

#[repr(C)]
pub enum MouseState {
    UpUp,
    UpDown,
    UpDownNotFlag,
    DownUp,
    Chording,
    ChordingNotFlag,
    DownUpAfterChording,
    Undefined,
}

#[repr(C)]
pub enum GameBoardState {
    Ready,
    Playing,
    Loss,
    Win,
}

fn struct_board_to_vec_board(board: Board) -> Vec<Vec<i32>> {
    let rows_ptr = unsafe { slice::from_raw_parts(board.rows, board.n_row) };
    let mut b: Vec<Vec<i32>> = vec![];
    for i in 0..board.n_row {
        b.push(vec![]);
        let array = unsafe { slice::from_raw_parts(rows_ptr[i].cells, rows_ptr[i].n_column) };
        for j in 0..rows_ptr[i].n_column {
            b[i].push(array[j]);
        }
    }
    b
}

fn vec_board_to_struct_board(mut b: Vec<Vec<i32>>) -> Board {
    let mut board: Vec<Row> = vec![];
    let n_row = b.len();
    for i in 0..n_row {
        board.push(Row {
            cells: b[i].as_mut_ptr(),
            n_column: b[0].len(),
        });
    }
    let p = board.as_mut_ptr();
    mem::forget(b);
    mem::forget(board);
    Board {
        rows: p,
        n_row: n_row,
    }
}

#[no_mangle]
pub extern "C" fn cal3BV(board: Board) -> usize {
    // assert!(!board.rows.is_null());
    // let rows_ptr = unsafe { slice::from_raw_parts(board.rows, board.n_row) };
    // let mut b: Vec<Vec<i32>> = vec![];
    // for i in 0..board.n_row {
    //     b.push(vec![]);
    //     let array = unsafe { slice::from_raw_parts(rows_ptr[i].cells, rows_ptr[i].n_column) };
    //     for j in 0..rows_ptr[i].n_column {
    //         b[i].push(array[j]);
    //     }
    // }
    rs_cal3BV(&struct_board_to_vec_board(board))
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
    // let mut board: Vec<Row> = vec![];
    // for i in 0..b.len() {
    //     board.push(Row {
    //         cells: b[i].as_mut_ptr(),
    //         n_column: column,
    //     });
    // }
    // let p = board.as_mut_ptr();
    // mem::forget(b);
    // mem::forget(board);
    // Board {
    //     rows: p,
    //     n_row: row,
    // }
    vec_board_to_struct_board(b)
}

#[no_mangle]
pub extern "C" fn cal_possibility_onboard(board_of_game: Board, mine_num: f64) -> BoardPossReturn {
    let rows_ptr = unsafe { slice::from_raw_parts(board_of_game.rows, board_of_game.n_row) };
    let mut b: Vec<Vec<i32>> = vec![];
    for i in 0..board_of_game.n_row {
        b.push(vec![]);
        let array = unsafe { slice::from_raw_parts(rows_ptr[i].cells, rows_ptr[i].n_column) };
        for j in 0..rows_ptr[i].n_column {
            b[i].push(array[j]);
        }
    }
    let (mut b, c) = rs_cal_possibility_onboard(&b, mine_num).unwrap();

    let mut board: Vec<RowPoss> = vec![];
    for i in 0..b.len() {
        board.push(RowPoss {
            cells_poss: b[i].as_mut_ptr(),
            n_column: b[0].len(),
        });
    }
    let p = board.as_mut_ptr();

    mem::forget(b);
    mem::forget(board);
    let t = BoardPoss {
        rows_poss: p,
        n_row: board_of_game.n_row,
    };
    BoardPossReturn {
        board_poss: t,
        min_mine_num: c[0],
        mine_num: c[1],
        max_mine_num: c[2],
    }
}

#[no_mangle]
pub extern "C" fn free_board(board: Board) {
    // 由rust分配的局面内存，也应该由rust释放
    unsafe {
        for i in 0..board.n_row {
            let layout =
                Layout::from_size_align_unchecked((*(board.rows)).n_column, mem::size_of::<i32>());
            dealloc((*(board.rows).offset(i as isize)).cells as *mut u8, layout);
        }
        let layout = Layout::from_size_align_unchecked(board.n_row as usize, mem::size_of::<Row>());
        dealloc(board.rows as *mut u8, layout);
    }
}

#[no_mangle]
pub extern "C" fn free_board_poss(board_poss: BoardPossReturn) {
    // 由rust分配的局面内存，也应该由rust释放
    unsafe {
        for i in 0..board_poss.board_poss.n_row {
            let layout = Layout::from_size_align_unchecked(
                (*(board_poss.board_poss.rows_poss)).n_column,
                mem::size_of::<f64>(),
            );
            dealloc(
                (*(board_poss.board_poss.rows_poss).offset(i as isize)).cells_poss as *mut u8,
                layout,
            );
        }
        let layout = Layout::from_size_align_unchecked(
            board_poss.board_poss.n_row,
            mem::size_of::<RowPoss>(),
        );
        dealloc(board_poss.board_poss.rows_poss as *mut u8, layout);
    }
}

// #[no_mangle]
// pub extern "C" fn minesweeperboard_new(board: Board) -> MinesweeperBoard {
//     let row = board.n_row;
//     let column = unsafe { (*(board.rows)).n_column };
//     MinesweeperBoard {
//         board,
//         row,
//         column,
//         game_board: vec_board_to_struct_board(vec![vec![10; column]; row]),
//         left: 0,
//         right: 0,
//         chording: 0,
//         ces: 0,
//         flag: 0,
//         solved3BV: 0,
//         flagedList: ptr::null_mut() as *mut Pointer,
//         mouse_state: MouseState::UpUp,
//         game_board_state: GameBoardState::Ready,
//         pointer_x: 0,
//         pointer_y: 0,
//     }
// }

// #[no_mangle]
// pub extern "C" fn minesweeperboard_step(_self: *mut MinesweeperBoard, _e: *const c_char, pos: Pointer) -> u8 {
//     let c_str: &CStr = unsafe { CStr::from_ptr(_e) };
//     let str_slice: &str = c_str.to_str().unwrap();

//     unsafe {(*_self).step()}
// }




