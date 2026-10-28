// 定向测试：问号标记（17）与 "l"/"r" 事件归一化
use ms_toollib::{GameBoardState, MinesweeperBoard, MouseState};

/// 4x4，雷在 (0,1)、(2,2)
fn guess_board() -> Vec<Vec<i32>> {
    vec![
        vec![1, -1, 1, 0],
        vec![1, 2, 2, 1],
        vec![0, 1, -1, 1],
        vec![0, 1, 1, 1],
    ]
}

/// guess_board 从 (0,3) 首次点击后的局面
fn guess_gb() -> Vec<Vec<i32>> {
    vec![
        vec![10, 10, 1, 0],
        vec![10, 10, 2, 1],
        vec![10, 10, 10, 10],
        vec![10, 10, 10, 10],
    ]
}

/// 4x4，雷在 (1,0)、(1,3)、(2,3)
fn chain_board() -> Vec<Vec<i32>> {
    vec![
        vec![1, 1, 1, 1],
        vec![-1, 1, 2, -1],
        vec![1, 1, 2, -1],
        vec![0, 0, 1, 1],
    ]
}

fn playing(
    board: Vec<Vec<i32>>,
    gb: Vec<Vec<i32>>,
    use_question: bool,
) -> MinesweeperBoard<Vec<Vec<i32>>> {
    let mut b = MinesweeperBoard::<Vec<Vec<i32>>>::new(board);
    b.use_question = use_question;
    b.game_board = gb;
    b.game_board_state = GameBoardState::Playing;
    b
}

fn rc(b: &mut MinesweeperBoard<Vec<Vec<i32>>>, pos: (usize, usize)) {
    b.step("rc", pos).unwrap();
    b.step("rr", pos).unwrap();
}

#[test]
fn question_cycle_when_enabled() {
    let mut b = playing(guess_board(), guess_gb(), true);
    // (2,2) 是雷：10→11→17→10→11
    rc(&mut b, (2, 2));
    assert_eq!(b.game_board[2][2], 11);
    assert_eq!(b.flag, 1);
    assert_eq!(b.rce, 1);
    rc(&mut b, (2, 2));
    assert_eq!(b.game_board[2][2], 17);
    assert_eq!(b.flag, 0);
    assert_eq!(b.rce, 1); // 问号转移不算 ce
    rc(&mut b, (2, 2));
    assert_eq!(b.game_board[2][2], 10);
    assert_eq!(b.flag, 0);
    rc(&mut b, (2, 2));
    assert_eq!(b.game_board[2][2], 11);
    assert_eq!(b.rce, 1); // 重复标同一个雷不重复计 ce
    // 非雷格同样的循环
    rc(&mut b, (0, 0));
    assert_eq!(b.game_board[0][0], 11);
    rc(&mut b, (0, 0));
    assert_eq!(b.game_board[0][0], 17);
    rc(&mut b, (0, 0));
    assert_eq!(b.game_board[0][0], 10);
}

#[test]
fn question_cycle_when_disabled() {
    let mut b = playing(guess_board(), guess_gb(), false);
    rc(&mut b, (2, 2));
    assert_eq!(b.game_board[2][2], 11);
    rc(&mut b, (2, 2));
    assert_eq!(b.game_board[2][2], 10); // 关问号：直接退标，没有 17
    assert_eq!(b.flag, 0);
}

#[test]
fn chord_skips_question_cell() {
    let mut gb = vec![vec![10; 4]; 4];
    gb[1][1] = 1;
    gb[1][0] = 11; // 旗
    gb[0][0] = 17; // 问号
    let mut b = playing(chain_board(), gb, true);
    b.step("cc", (1, 1)).unwrap();
    b.step("lr", (1, 1)).unwrap();
    assert_ne!(b.game_board_state, GameBoardState::Loss);
    // 10 的邻格被双击打开
    assert!(b.game_board[0][1] < 10);
    assert!(b.game_board[2][1] < 10);
    // 问号格不参与双击：不被打开，也不阻挡
    assert_eq!(b.game_board[0][0], 17);
    assert_eq!(b.double, 1);
}

#[test]
fn l_r_click_in_ready() {
    let mut b = MinesweeperBoard::<Vec<Vec<i32>>>::new(guess_board());
    b.step("l", (0, 3)).unwrap(); // 按下
    b.step("l", (0, 3)).unwrap(); // 抬起，打开
    assert_eq!(b.game_board[0][3], 0);
    assert_eq!(b.left, 1);
    assert_eq!(b.game_board_state, GameBoardState::Playing);
}

#[test]
fn l_r_flag_in_ready_then_click() {
    let mut b = MinesweeperBoard::<Vec<Vec<i32>>>::new(guess_board());
    b.step("r", (0, 0)).unwrap(); // 预标雷
    assert_eq!(b.game_board[0][0], 11);
    b.step("r", (0, 0)).unwrap(); // 右键抬起
    b.step("l", (0, 3)).unwrap();
    b.step("l", (0, 3)).unwrap();
    assert_eq!(b.game_board[0][3], 0);
    assert_eq!(b.game_board[0][0], 11); // 预标的雷保留
    assert_eq!(b.game_board_state, GameBoardState::Playing);
}

#[test]
fn l_r_chord_in_playing() {
    let mut gb = vec![vec![10; 4]; 4];
    gb[1][1] = 1;
    gb[1][0] = 11;
    let mut b = playing(chain_board(), gb, false);
    b.step("l", (1, 1)).unwrap(); // 左键按下
    b.step("r", (1, 1)).unwrap(); // 右键按下，进入双击
    b.step("l", (1, 1)).unwrap(); // 左键抬起，执行双击
    assert!(b.game_board[0][1] < 10);
    assert_eq!(b.double, 1);
    assert_eq!(b.dce, 1);
}

#[test]
fn question_preflag_cycle_in_ready() {
    // 预标雷阶段开问号：10→11→17→10，旗数清零后回到 Ready
    let mut b = MinesweeperBoard::<Vec<Vec<i32>>>::new(guess_board());
    b.use_question = true;
    b.step("rc", (0, 0)).unwrap();
    assert_eq!(b.game_board_state, GameBoardState::PreFlaging);
    b.step("rr", (0, 0)).unwrap();
    b.step("rc", (0, 0)).unwrap();
    assert_eq!(b.game_board[0][0], 17);
    b.step("rr", (0, 0)).unwrap();
    b.step("rc", (0, 0)).unwrap();
    assert_eq!(b.game_board[0][0], 10);
    b.step("rr", (0, 0)).unwrap();
    assert_eq!(b.game_board_state, GameBoardState::Ready);
    // 仍然可以正常开局
    b.step("lc", (0, 3)).unwrap();
    b.step("lr", (0, 3)).unwrap();
    assert_eq!(b.game_board[0][3], 0);
    assert_eq!(b.game_board_state, GameBoardState::Playing);
}

#[test]
fn release_on_flag_in_preflaging_restores_mouse_state() {
    // 复现 temp.avf：预标雷阶段左键按下后在旗上弹起，之后右击必须能正常退标
    let mut b = MinesweeperBoard::<Vec<Vec<i32>>>::new(guess_board());
    b.step("rc", (0, 0)).unwrap();
    b.step("rr", (0, 0)).unwrap();
    assert_eq!(b.game_board[0][0], 11);
    assert_eq!(b.game_board_state, GameBoardState::PreFlaging);
    b.step("lc", (0, 1)).unwrap(); // 左键按下
    b.step("lr", (0, 0)).unwrap(); // 在旗上弹起
    assert_eq!(b.mouse_state, MouseState::UpUp);
    // 右击退标，旗数清零后回到 Ready
    b.step("rc", (0, 0)).unwrap();
    assert_eq!(b.game_board[0][0], 10);
    assert_eq!(b.flag, 0);
    b.step("rr", (0, 0)).unwrap();
    assert_eq!(b.game_board_state, GameBoardState::Ready);
    // 随后仍可正常开局，未被误判为双击
    b.step("lc", (0, 3)).unwrap();
    b.step("lr", (0, 3)).unwrap();
    assert_eq!(b.game_board[0][3], 0);
    assert_eq!(b.game_board_state, GameBoardState::Playing);
    assert_eq!(b.double, 0);
}

#[test]
fn m_chord_in_playing() {
    // Playing 阶段 "m" 按下+"m" 抬起 = 一次中键双击
    let mut gb = vec![vec![10; 4]; 4];
    gb[1][1] = 1;
    gb[1][0] = 11;
    let mut b = playing(chain_board(), gb, false);
    b.step("m", (1, 1)).unwrap(); // 中键按下 → mc
    b.step("m", (1, 1)).unwrap(); // 中键抬起 → mr，执行双击
    assert!(b.game_board[0][1] < 10);
    assert_eq!(b.double, 1);
    assert_eq!(b.dce, 1);
}

#[test]
fn m_hold_from_ready_to_playing() {
    // 中键在 Ready 阶段按下（arbiter 录像可能如此），middle_hold 跨阶段保持，
    // Playing 阶段再收到 "m" 应翻译为抬起并触发双击
    let mut b = MinesweeperBoard::<Vec<Vec<i32>>>::new(chain_board());
    b.step("m", (1, 1)).unwrap(); // 按下
    b.step("lc", (1, 1)).unwrap();
    b.step("lr", (1, 1)).unwrap(); // 开局，(1,1)=1
    assert_eq!(b.game_board_state, GameBoardState::Playing);
    b.step("rc", (1, 0)).unwrap();
    b.step("rr", (1, 0)).unwrap(); // 标雷 (1,0)
    b.step("m", (1, 1)).unwrap(); // 抬起 → mr → 双击
    assert_eq!(b.double, 1);
    assert!(b.game_board[0][0] < 10); // 邻格被双击打开
}
