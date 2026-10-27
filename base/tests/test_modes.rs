// 定向测试：局面状态机内置的 EVF 模式（6 / 9 / 10 / 11–16）
use ms_toollib::{GameBoardState, MinesweeperBoard};

/// 4x4，雷在 (0,1)、(2,2)。
/// 从 (0,3) 开始扫开一片区域后，局面仍有歧义（不可解）。
fn guess_board() -> Vec<Vec<i32>> {
    vec![
        vec![1, -1, 1, 0],
        vec![1, 2, 2, 1],
        vec![0, 1, -1, 1],
        vec![0, 1, 1, 1],
    ]
}

/// 上一局面在 (0,3) 首次点击后的局面
fn guess_game_board() -> Vec<Vec<i32>> {
    vec![
        vec![10, 10, 1, 0],
        vec![10, 10, 2, 1],
        vec![10, 10, 10, 10],
        vec![10, 10, 10, 10],
    ]
}

/// 4x4，雷在 (1,0)、(1,3)、(2,3)，用于递归双击的链式场景
fn chain_board() -> Vec<Vec<i32>> {
    vec![
        vec![1, 1, 1, 1],
        vec![-1, 1, 2, -1],
        vec![1, 1, 2, -1],
        vec![0, 0, 1, 1],
    ]
}

fn board_of(mode: u16, board: Vec<Vec<i32>>, game_board: Vec<Vec<i32>>) -> MinesweeperBoard<Vec<Vec<i32>>> {
    let mut b = MinesweeperBoard::<Vec<Vec<i32>>>::new(board);
    b.set_mode(mode);
    b.game_board = game_board;
    b.game_board_state = GameBoardState::Playing;
    b
}

fn mine_count(board: &Vec<Vec<i32>>) -> usize {
    board
        .iter()
        .map(|row| row.iter().filter(|&&v| v == -1).count())
        .sum()
}

fn click(b: &mut MinesweeperBoard<Vec<Vec<i32>>>, pos: (usize, usize)) {
    b.step("lc", pos).unwrap();
    b.step("lr", pos).unwrap();
}

#[test]
fn mode_6_first_click_is_exempt() {
    let mut b = MinesweeperBoard::<Vec<Vec<i32>>>::new(guess_board());
    b.set_mode(6);
    click(&mut b, (0, 3));
    assert_ne!(b.game_board_state, GameBoardState::Loss);
    assert_eq!(b.game_board[0][3], 0);
}

#[test]
fn mode_6_relayout_to_mine_loses() {
    let mut b = board_of(6, guess_board(), guess_game_board());
    click(&mut b, (1, 1));
    assert_eq!(b.game_board_state, GameBoardState::Loss);
    assert_eq!(b.board[1][1], -1);
    assert!(b.board_changed);
    assert_eq!(mine_count(&b.board), 2);
}

#[test]
fn mode_6_determined_safe_opens_normally() {
    let mut b = board_of(6, guess_board(), guess_game_board());
    // (2,1) 已被判雷引擎确定为非雷，无法重排成雷
    click(&mut b, (2, 1));
    assert_ne!(b.game_board_state, GameBoardState::Loss);
    assert_eq!(b.game_board[2][1], 1);
    assert!(!b.board_changed);
}

#[test]
fn mode_9_relayout_keeps_solvable_and_no_loss() {
    let mut b = board_of(9, guess_board(), guess_game_board());
    click(&mut b, (2, 2));
    assert_ne!(b.game_board_state, GameBoardState::Loss);
    assert_eq!(b.board[2][2], 1);
    assert!(b.board_changed);
    assert_eq!(mine_count(&b.board), 2);
}

#[test]
fn mode_9_determined_mine_loses() {
    // 除雷以外的格子全部打开，只剩下两个雷未打开，任何一致的雷局都不能让 (2,2) 变成非雷
    let board = guess_board();
    let mut gb = vec![vec![10; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            if board[i][j] != -1 {
                gb[i][j] = board[i][j];
            }
        }
    }
    let mut b = board_of(9, board, gb);
    click(&mut b, (2, 2));
    assert_eq!(b.game_board_state, GameBoardState::Loss);
    assert!(!b.board_changed);
}

#[test]
fn mode_10_relayout_without_solvable_requirement() {
    let mut b = board_of(10, guess_board(), guess_game_board());
    click(&mut b, (2, 2));
    assert_ne!(b.game_board_state, GameBoardState::Loss);
    assert_eq!(b.board[2][2], 1);
    assert!(b.board_changed);
    assert_eq!(mine_count(&b.board), 2);
}

#[test]
fn mode_10_determined_mine_loses() {
    let board = guess_board();
    let mut gb = vec![vec![10; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            if board[i][j] != -1 {
                gb[i][j] = board[i][j];
            }
        }
    }
    let mut b = board_of(10, board, gb);
    click(&mut b, (2, 2));
    assert_eq!(b.game_board_state, GameBoardState::Loss);
}

/// 链式双击的局面：(1,1)=1 已打开，三个雷已标出
fn chain_game_board() -> Vec<Vec<i32>> {
    vec![
        vec![10, 10, 10, 10],
        vec![11, 1, 10, 11],
        vec![10, 10, 10, 11],
        vec![10, 10, 10, 10],
    ]
}

fn chord_at(b: &mut MinesweeperBoard<Vec<Vec<i32>>>, pos: (usize, usize)) {
    b.step("cc", pos).unwrap();
    b.step("lr", pos).unwrap();
}

#[test]
fn mode_11_chord_recursion() {
    let mut b = board_of(11, chain_board(), chain_game_board());
    chord_at(&mut b, (1, 1));
    assert_ne!(b.game_board_state, GameBoardState::Loss);
    // (0,3) 不与 (1,1) 相邻，只能由连锁静默双击打开
    assert!(b.game_board[0][3] < 10);
    // 玩家只做了一次双击；连锁的自动双击不计数
    assert_eq!(b.double, 1);
    assert_eq!(b.dce, 1);
}

#[test]
fn mode_11_without_mode_does_not_recurse() {
    let mut b = board_of(0, chain_board(), chain_game_board());
    chord_at(&mut b, (1, 1));
    assert_eq!(b.game_board_state, GameBoardState::Playing);
    assert_eq!(b.game_board[0][3], 10);
}

/// 标雷连锁的局面：(1,1)=1 已打开，其余未打开
fn flag_game_board() -> Vec<Vec<i32>> {
    let mut gb = vec![vec![10; 4]; 4];
    gb[1][1] = 1;
    gb
}

fn flag_at(b: &mut MinesweeperBoard<Vec<Vec<i32>>>, pos: (usize, usize)) {
    b.step("rc", pos).unwrap();
    b.step("rr", pos).unwrap();
}

#[test]
fn mode_12_flag_recursion() {
    let mut b = board_of(12, chain_board(), flag_game_board());
    flag_at(&mut b, (1, 0));
    assert_ne!(b.game_board_state, GameBoardState::Loss);
    // 标雷后，(1,1) 周围的格子被自动双击打开
    assert!(b.game_board[1][2] < 10);
    assert_eq!(b.right, 1);
    assert_eq!(b.rce, 1);
    // 自动双击不计入玩家双击次数
    assert_eq!(b.double, 0);
    assert_eq!(b.dce, 0);
}

#[test]
fn mode_13_both_recursions() {
    let mut chained = board_of(13, chain_board(), chain_game_board());
    chord_at(&mut chained, (1, 1));
    assert!(chained.game_board[0][3] < 10);
    assert_eq!(chained.double, 1);
    assert_eq!(chained.dce, 1);

    let mut flagged = board_of(13, chain_board(), flag_game_board());
    flag_at(&mut flagged, (1, 0));
    assert!(flagged.game_board[1][2] < 10);
    assert_eq!(flagged.double, 0);
    assert_eq!(flagged.dce, 0);
}

#[test]
fn mode_14_guess_and_chord_recursion() {
    // 可猜重排 + 双击递归
    let mut guess = board_of(14, guess_board(), guess_game_board());
    click(&mut guess, (2, 2));
    assert_ne!(guess.game_board_state, GameBoardState::Loss);
    assert_eq!(guess.board[2][2], 1);

    let mut chained = board_of(14, chain_board(), chain_game_board());
    chord_at(&mut chained, (1, 1));
    assert!(chained.game_board[0][3] < 10);
    assert_eq!(chained.dce, 1);
}

#[test]
fn mode_15_guess_and_flag_recursion() {
    let mut guess = board_of(15, guess_board(), guess_game_board());
    click(&mut guess, (2, 2));
    assert_ne!(guess.game_board_state, GameBoardState::Loss);
    assert_eq!(guess.board[2][2], 1);

    let mut flagged = board_of(15, chain_board(), flag_game_board());
    flag_at(&mut flagged, (1, 0));
    assert!(flagged.game_board[1][2] < 10);
    assert_eq!(flagged.dce, 0);
}

#[test]
fn mode_16_guess_and_both_recursions() {
    let mut guess = board_of(16, guess_board(), guess_game_board());
    click(&mut guess, (2, 2));
    assert_ne!(guess.game_board_state, GameBoardState::Loss);
    assert_eq!(guess.board[2][2], 1);

    let mut chained = board_of(16, chain_board(), chain_game_board());
    chord_at(&mut chained, (1, 1));
    assert!(chained.game_board[0][3] < 10);

    let mut flagged = board_of(16, chain_board(), flag_game_board());
    flag_at(&mut flagged, (1, 0));
    assert!(flagged.game_board[1][2] < 10);
}
