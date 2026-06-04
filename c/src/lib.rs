extern crate libc;
use ms_toollib::cal_bbbv as rs_cal_bbbv;
use ms_toollib::cal_hzini as rs_cal_hzini;
use ms_toollib::cal_isl as rs_cal_isl;
use ms_toollib::cal_op as rs_cal_op;
use ms_toollib::cal_probability_onboard as rs_cal_probability_onboard;
use ms_toollib::cal_zini as rs_cal_zini;
use ms_toollib::cal_rzini as rs_cal_rzini;
use ms_toollib::laymine as rs_laymine;
use std::alloc::{dealloc, Layout};
use std::os::raw::c_void;
use std::ffi::CString;
use std::mem;
use std::slice;

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
pub extern "C" fn cal_bbbv(board: Board) -> usize {
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
    rs_cal_bbbv(&struct_board_to_vec_board(board))
}

#[no_mangle]
pub extern "C" fn laymine(
    row: usize,
    column: usize,
    MineNum: usize,
    X0: usize,
    Y0: usize,
) -> Board {
    let b = rs_laymine(row, column, MineNum, X0, Y0);
    // let mut b = rs_laymine(row, column, MineNum, X0, Y0);
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
pub extern "C" fn cal_probability_onboard(board_of_game: Board, mine_num: f64) -> BoardPossReturn {
    let rows_ptr = unsafe { slice::from_raw_parts(board_of_game.rows, board_of_game.n_row) };
    let mut b: Vec<Vec<i32>> = vec![];
    for i in 0..board_of_game.n_row {
        b.push(vec![]);
        let array = unsafe { slice::from_raw_parts(rows_ptr[i].cells, rows_ptr[i].n_column) };
        for j in 0..rows_ptr[i].n_column {
            b[i].push(array[j]);
        }
    }
    let (mut b, c) = rs_cal_probability_onboard(&b, mine_num).unwrap();

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
pub extern "C" fn cal_zini(board: Board) -> usize {
    rs_cal_zini(&struct_board_to_vec_board(board))
}

#[no_mangle]
pub extern "C" fn cal_hzini(board: Board) -> usize {
    rs_cal_hzini(&struct_board_to_vec_board(board))
}

#[no_mangle]
pub extern "C" fn cal_rzini(board: Board, n_iter: usize) -> usize {
    rs_cal_rzini(&struct_board_to_vec_board(board), n_iter)
}

#[no_mangle]
pub extern "C" fn cal_isl(board: Board) -> usize {
    rs_cal_isl(&struct_board_to_vec_board(board))
}

#[no_mangle]
pub extern "C" fn cal_op(board: Board) -> usize {
    rs_cal_op(&struct_board_to_vec_board(board))
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

// ──────────── Video type FFI ────────────

use ms_toollib::videos::*;
use std::ffi::CStr;
use std::os::raw::c_char;

use ms_toollib::AvfVideo as RsAvf;
use ms_toollib::EvfVideo as RsEvf;
use ms_toollib::MvfVideo as RsMvf;
use ms_toollib::RmvVideo as RsRmv;
use ms_toollib::BaseVideo as RsBaseVideo;

// Helper: C string → &str (panics on failure)
unsafe fn cstr_to_str<'a>(p: *const c_char) -> &'a str {
    CStr::from_ptr(p).to_str().unwrap()
}

macro_rules! new_from_file {
    ($ty:ty, $ptr:expr) => {{
        let s = unsafe { cstr_to_str($ptr) };
        Box::into_raw(Box::new(<$ty as NewSomeVideo<&str>>::new(s))) as *mut c_void
    }};
}

macro_rules! new_from_data {
    ($ty:ty, $data:expr, $len:expr, $fname:expr) => {{
        let raw = unsafe { std::slice::from_raw_parts($data, $len) }.to_vec();
        let s = unsafe { cstr_to_str($fname) };
        Box::into_raw(Box::new(<$ty as NewSomeVideo2<Vec<u8>, &str>>::new(raw, s))) as *mut c_void
    }};
}

// ─── AvfVideo ───

#[no_mangle]
pub extern "C" fn avf_video_new(filename: *const c_char) -> *mut c_void { new_from_file!(RsAvf, filename) }
#[no_mangle]
pub extern "C" fn avf_video_new_from_data(data: *const u8, len: usize, filename: *const c_char) -> *mut c_void { new_from_data!(RsAvf, data, len, filename) }
#[no_mangle]
pub extern "C" fn avf_video_free(ptr: *mut c_void) { if !ptr.is_null() { unsafe { drop(Box::from_raw(ptr as *mut RsAvf)); } } }
#[no_mangle]
pub extern "C" fn avf_video_parse(ptr: *mut c_void) -> i32 {
    let v = unsafe { &mut *(ptr as *mut RsAvf) };
    match v.parse() { Ok(()) => 0, Err(_) => -1 }
}
#[no_mangle]
pub extern "C" fn avf_video_data_ptr(ptr: *mut c_void) -> *mut c_void {
    let v = unsafe { &mut *(ptr as *mut RsAvf) };
    &mut v.data as *mut RsBaseVideo<Vec<Vec<i32>>> as *mut c_void
}

// ─── EvfVideo ───

#[no_mangle]
pub extern "C" fn evf_video_new(filename: *const c_char) -> *mut c_void { new_from_file!(RsEvf, filename) }
#[no_mangle]
pub extern "C" fn evf_video_new_from_data(data: *const u8, len: usize, filename: *const c_char) -> *mut c_void { new_from_data!(RsEvf, data, len, filename) }
#[no_mangle]
pub extern "C" fn evf_video_free(ptr: *mut c_void) { if !ptr.is_null() { unsafe { drop(Box::from_raw(ptr as *mut RsEvf)); } } }
#[no_mangle]
pub extern "C" fn evf_video_parse(ptr: *mut c_void) -> i32 {
    let v = unsafe { &mut *(ptr as *mut RsEvf) };
    match v.parse() { Ok(()) => 0, Err(_) => -1 }
}
#[no_mangle]
pub extern "C" fn evf_video_data_ptr(ptr: *mut c_void) -> *mut c_void {
    let v = unsafe { &mut *(ptr as *mut RsEvf) };
    &mut v.data as *mut RsBaseVideo<Vec<Vec<i32>>> as *mut c_void
}

// ─── MvfVideo ───

#[no_mangle]
pub extern "C" fn mvf_video_new(filename: *const c_char) -> *mut c_void { new_from_file!(RsMvf, filename) }
#[no_mangle]
pub extern "C" fn mvf_video_new_from_data(data: *const u8, len: usize, filename: *const c_char) -> *mut c_void { new_from_data!(RsMvf, data, len, filename) }
#[no_mangle]
pub extern "C" fn mvf_video_free(ptr: *mut c_void) { if !ptr.is_null() { unsafe { drop(Box::from_raw(ptr as *mut RsMvf)); } } }
#[no_mangle]
pub extern "C" fn mvf_video_parse(ptr: *mut c_void) -> i32 {
    let v = unsafe { &mut *(ptr as *mut RsMvf) };
    match v.parse() { Ok(()) => 0, Err(_) => -1 }
}
#[no_mangle]
pub extern "C" fn mvf_video_data_ptr(ptr: *mut c_void) -> *mut c_void {
    let v = unsafe { &mut *(ptr as *mut RsMvf) };
    &mut v.data as *mut RsBaseVideo<Vec<Vec<i32>>> as *mut c_void
}

// ─── RmvVideo ───

#[no_mangle]
pub extern "C" fn rmv_video_new(filename: *const c_char) -> *mut c_void { new_from_file!(RsRmv, filename) }
#[no_mangle]
pub extern "C" fn rmv_video_new_from_data(data: *const u8, len: usize, filename: *const c_char) -> *mut c_void { new_from_data!(RsRmv, data, len, filename) }
#[no_mangle]
pub extern "C" fn rmv_video_free(ptr: *mut c_void) { if !ptr.is_null() { unsafe { drop(Box::from_raw(ptr as *mut RsRmv)); } } }
#[no_mangle]
pub extern "C" fn rmv_video_parse(ptr: *mut c_void) -> i32 {
    let v = unsafe { &mut *(ptr as *mut RsRmv) };
    match v.parse() { Ok(()) => 0, Err(_) => -1 }
}
#[no_mangle]
pub extern "C" fn rmv_video_data_ptr(ptr: *mut c_void) -> *mut c_void {
    let v = unsafe { &mut *(ptr as *mut RsRmv) };
    &mut v.data as *mut RsBaseVideo<Vec<Vec<i32>>> as *mut c_void
}

// ─── BaseVideo<Vec<Vec<i32>>> common operations ───

macro_rules! bv { ($ptr:expr) => { unsafe { &mut *($ptr as *mut BaseVideo<Vec<Vec<i32>>>) } } }

#[no_mangle]
pub extern "C" fn base_video_analyse(ptr: *mut c_void) {
    bv!(ptr).analyse();
}

#[no_mangle]
pub extern "C" fn base_video_get_rtime(ptr: *mut c_void) -> f64 {
    bv!(ptr).get_rtime().unwrap_or(0.0)
}

#[no_mangle]
pub extern "C" fn base_video_get_rtime_ms(ptr: *mut c_void) -> u32 {
    bv!(ptr).get_rtime_ms().unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn base_video_get_etime(ptr: *mut c_void) -> f64 {
    bv!(ptr).get_etime().unwrap_or(0.0)
}

#[no_mangle]
pub extern "C" fn base_video_get_game_board(ptr: *mut c_void) -> Board {
    let gb = bv!(ptr).get_game_board();
    vec_board_to_struct_board(gb)
}

#[no_mangle]
pub extern "C" fn base_video_get_left(ptr: *mut c_void) -> usize { bv!(ptr).get_left() }
#[no_mangle]
pub extern "C" fn base_video_get_right(ptr: *mut c_void) -> usize { bv!(ptr).get_right() }
#[no_mangle]
pub extern "C" fn base_video_get_double(ptr: *mut c_void) -> usize { bv!(ptr).get_double() }
#[no_mangle]
pub extern "C" fn base_video_get_cl(ptr: *mut c_void) -> usize { bv!(ptr).get_cl() }
#[no_mangle]
pub extern "C" fn base_video_get_flag(ptr: *mut c_void) -> usize { bv!(ptr).get_flag() }
#[no_mangle]
pub extern "C" fn base_video_get_bbbv_solved(ptr: *mut c_void) -> usize { bv!(ptr).get_bbbv_solved().unwrap_or(0) }
#[no_mangle]
pub extern "C" fn base_video_get_ce(ptr: *mut c_void) -> usize { bv!(ptr).get_ce().unwrap_or(0) }
#[no_mangle]
pub extern "C" fn base_video_get_corr(ptr: *mut c_void) -> f64 { bv!(ptr).get_corr().unwrap_or(0.0) }
#[no_mangle]
pub extern "C" fn base_video_get_thrp(ptr: *mut c_void) -> f64 { bv!(ptr).get_thrp().unwrap_or(0.0) }
#[no_mangle]
pub extern "C" fn base_video_get_ioe(ptr: *mut c_void) -> f64 { bv!(ptr).get_ioe().unwrap_or(0.0) }
#[no_mangle]
pub extern "C" fn base_video_get_path(ptr: *mut c_void) -> f64 { bv!(ptr).get_path() }
#[no_mangle]
pub extern "C" fn base_video_get_stnb(ptr: *mut c_void) -> f64 { bv!(ptr).get_stnb().unwrap_or(0.0) }
#[no_mangle]
pub extern "C" fn base_video_get_mouse_state(ptr: *mut c_void) -> usize { bv!(ptr).get_mouse_state() }
#[no_mangle]
pub extern "C" fn base_video_get_current_event_id(ptr: *mut c_void) -> usize { bv!(ptr).current_event_id }
#[no_mangle]
pub extern "C" fn base_video_set_current_event_id(ptr: *mut c_void, id: usize) -> u8 {
    bv!(ptr).set_current_event_id(id).unwrap_or(0)
}
#[no_mangle]
pub extern "C" fn base_video_get_current_time(ptr: *mut c_void) -> f64 { bv!(ptr).current_time }
#[no_mangle]
pub extern "C" fn base_video_set_current_time(ptr: *mut c_void, t: f64) { bv!(ptr).set_current_time(t) }
#[no_mangle]
pub extern "C" fn base_video_get_event_count(ptr: *mut c_void) -> usize { bv!(ptr).video_action_state_recorder.len() }
#[no_mangle]
pub extern "C" fn base_video_is_valid(ptr: *mut c_void) -> u8 { bv!(ptr).is_valid() }

// Event accessors
#[no_mangle]
pub extern "C" fn base_video_event_time(ptr: *mut c_void, idx: usize) -> f64 {
    let bv = bv!(ptr);
    if idx < bv.video_action_state_recorder.len() {
        bv.video_action_state_recorder[idx].time
    } else { -1.0 }
}

#[no_mangle]
pub extern "C" fn base_video_event_desc(ptr: *mut c_void, idx: usize) -> *mut c_char {
    let bv = bv!(ptr);
    if idx < bv.video_action_state_recorder.len() {
        let desc = match &bv.video_action_state_recorder[idx].event {
            Some(Event::Mouse(me)) => format!("{},{},{}", me.mouse, me.x, me.y),
            Some(Event::GameState(gs)) => gs.game_state.clone(),
            Some(Event::Board(be)) => format!("{},{},{}", be.board, be.row_id, be.column_id),
            Some(Event::Index(ie)) => format!("{}/{}", ie.key, match &ie.value {
                IndexValue::Number(n) => n.to_string(),
                IndexValue::String(s) => s.clone(),
            }),
            None => "None".to_string(),
        };
        let s = CString::new(desc).unwrap();
        s.into_raw() as *mut c_char
    } else { std::ptr::null_mut() }
}

#[no_mangle]
pub extern "C" fn base_video_free_event_desc(s: *mut c_char) {
    if !s.is_null() { unsafe { drop(CString::from_raw(s)); } }
}

// Field getters
#[no_mangle]
pub extern "C" fn base_video_get_software(ptr: *mut c_void) -> *mut c_char {
    CString::new(bv!(ptr).software.as_str()).unwrap().into_raw() as *mut c_char
}
#[no_mangle]
pub extern "C" fn base_video_get_player(ptr: *mut c_void) -> *mut c_char {
    CString::new(bv!(ptr).player_identifier.as_str()).unwrap().into_raw() as *mut c_char
}
#[no_mangle]
pub extern "C" fn base_video_free_string(s: *mut c_char) {
    if !s.is_null() { unsafe { drop(CString::from_raw(s)); } }
}

#[no_mangle]
pub extern "C" fn base_video_get_width(ptr: *mut c_void) -> usize { bv!(ptr).width }
#[no_mangle]
pub extern "C" fn base_video_get_height(ptr: *mut c_void) -> usize { bv!(ptr).height }
#[no_mangle]
pub extern "C" fn base_video_get_mine_num(ptr: *mut c_void) -> usize { bv!(ptr).mine_num }
#[no_mangle]
pub extern "C" fn base_video_get_mode(ptr: *mut c_void) -> u16 { bv!(ptr).mode }
#[no_mangle]
pub extern "C" fn base_video_get_level(ptr: *mut c_void) -> u8 { bv!(ptr).level }
#[no_mangle]
pub extern "C" fn base_video_get_nf(ptr: *mut c_void) -> u8 { bv!(ptr).nf as u8 }
#[no_mangle]
pub extern "C" fn base_video_get_is_completed(ptr: *mut c_void) -> u8 { bv!(ptr).is_completed as u8 }


