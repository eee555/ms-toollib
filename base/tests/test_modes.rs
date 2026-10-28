// 定向测试：局面状态机内置的 EVF 模式（6 / 7 / 8 / 9 / 10 / 11–16）
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
    assert_ne!(b.board[2][2], -1);
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
    assert_ne!(b.board[2][2], -1);
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
    assert_ne!(guess.board[2][2], -1);

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
    assert_ne!(guess.board[2][2], -1);

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
    assert_ne!(guess.board[2][2], -1);

    let mut chained = board_of(16, chain_board(), chain_game_board());
    chord_at(&mut chained, (1, 1));
    assert!(chained.game_board[0][3] < 10);

    let mut flagged = board_of(16, chain_board(), flag_game_board());
    flag_at(&mut flagged, (1, 0));
    assert!(flagged.game_board[1][2] < 10);
}

/// 除 (2,1) 以外的非雷格全部打开：(2,1) 由引擎判定为一定不是雷，局面可解
fn solved_except_game_board() -> Vec<Vec<i32>> {
    let board = guess_board();
    let mut gb = vec![vec![10; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            if board[i][j] != -1 {
                gb[i][j] = board[i][j];
            }
        }
    }
    gb[2][1] = 10;
    gb
}

/// 3x5，雷在 (2,1)、(2,3)，只打开 (1,1)=1、(1,2)=2。
/// 两个约束互不包含，推不出任何必然非雷格，属于只能猜的局面。
fn must_guess_board() -> Vec<Vec<i32>> {
    vec![
        vec![0, 0, 0, 0, 0],
        vec![1, 1, 2, 1, 1],
        vec![1, -1, 2, -1, 1],
    ]
}

fn must_guess_game_board() -> Vec<Vec<i32>> {
    vec![
        vec![10, 10, 10, 10, 10],
        vec![10, 1, 2, 10, 10],
        vec![10, 10, 10, 10, 10],
    ]
}

#[test]
fn mode_7_must_guess_click_on_mine_is_safe() {
    // 局面里一个能用逻辑证明的安全格都没有：点真雷也不判负
    let mut b = board_of(7, must_guess_board(), must_guess_game_board());
    click(&mut b, (2, 1));
    assert_ne!(b.game_board_state, GameBoardState::Loss);
    assert_ne!(b.board[2][1], -1);
    assert!(b.board_changed);
    assert_eq!(mine_count(&b.board), 2);
}

#[test]
fn mode_7_needless_guess_loses() {
    // 局面里还有可推出的安全格 (2,1)，玩家却点了“可能是雷”的 (1,1)。
    // 该格实际不是雷，但重排成它是雷的一致雷局后照样判负
    let mut b = board_of(7, guess_board(), guess_game_board());
    assert_eq!(b.board[1][1], 2);
    click(&mut b, (1, 1));
    assert_eq!(b.game_board_state, GameBoardState::Loss);
    assert!(b.board_changed);
    assert_eq!(b.board[1][1], -1);
    assert_eq!(mine_count(&b.board), 2);
}

#[test]
fn mode_7_determined_safe_opens_normally() {
    // 该格一定不是雷：正常打开，不重排
    let mut b = board_of(7, guess_board(), solved_except_game_board());
    click(&mut b, (2, 1));
    assert_ne!(b.game_board_state, GameBoardState::Loss);
    assert_eq!(b.game_board[2][1], 1);
    assert!(!b.board_changed);
}

#[test]
fn mode_7_needless_guess_chord_loses() {
    // (2,2) 标了旗（真雷），双击 (1,3) 会打开实际不是雷、但“可能是雷”的 (2,3)
    let mut gb = guess_game_board();
    gb[2][2] = 11;
    let mut b = board_of(7, guess_board(), gb);
    b.flag = 1;
    chord_at(&mut b, (1, 3));
    assert_eq!(b.game_board_state, GameBoardState::Loss);
    assert!(b.board_changed);
}

#[test]
fn mode_7_must_guess_chord_does_not_lose() {
    // 只能猜的局面：双击会打开真雷 (2,1)，重排后这一击不判负
    let mut gb = must_guess_game_board();
    gb[0][2] = 11;
    let mut b = board_of(7, must_guess_board(), gb);
    b.flag = 1;
    chord_at(&mut b, (1, 1));
    assert_ne!(b.game_board_state, GameBoardState::Loss);
    assert!(b.board_changed);
}

#[test]
fn mode_8_must_guess_click_on_mine_is_safe() {
    // 准无猜：推不出任何安全格、只能猜时，点到真雷豁免，重排成非雷
    let mut b = board_of(8, must_guess_board(), must_guess_game_board());
    click(&mut b, (2, 1));
    assert_ne!(b.game_board_state, GameBoardState::Loss);
    assert_ne!(b.board[2][1], -1);
    assert!(b.board_changed);
    assert_eq!(mine_count(&b.board), 2);
}

#[test]
fn mode_8_solvable_click_on_mine_still_loses() {
    // 局面里还有可推出的安全格 (2,1)：点到真雷不豁免，直接判负
    let mut b = board_of(8, guess_board(), guess_game_board());
    click(&mut b, (2, 2));
    assert_eq!(b.game_board_state, GameBoardState::Loss);
    assert!(!b.board_changed);
}

#[test]
fn mode_8_must_guess_chord_does_not_lose() {
    // 只能猜的局面：双击会打开真雷 (2,1)，重排后这一击不判负
    let mut gb = must_guess_game_board();
    gb[0][2] = 11;
    let mut b = board_of(8, must_guess_board(), gb);
    b.flag = 1;
    chord_at(&mut b, (1, 1));
    assert_ne!(b.game_board_state, GameBoardState::Loss);
    assert!(b.board_changed);
    assert_eq!(mine_count(&b.board), 2);
}

#[test]
fn mode_8_solvable_chord_still_loses() {
    // 旗插错了位置：(1,1)、(2,3) 都不是雷，双击会把 (0,1)、(2,2) 两个雷区打开。
    // 局面还有可推出的安全格，不豁免，判负
    let mut gb = guess_game_board();
    gb[1][1] = 11;
    gb[2][3] = 11;
    let mut b = board_of(8, guess_board(), gb);
    b.flag = 2;
    chord_at(&mut b, (1, 2));
    assert_eq!(b.game_board_state, GameBoardState::Loss);
    assert!(!b.board_changed);
}
