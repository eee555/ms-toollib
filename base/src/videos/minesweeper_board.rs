use crate::utils::refresh_board;

#[cfg(any(feature = "py", feature = "rs"))]
use crate::safe_board::SafeBoard;

use crate::safe_board::{BoardSize, EmptyBoard, ReplaceBoard};
use crate::videos::relayout;

use std::cmp::{max, min};
use std::collections::VecDeque;

/// 可猜模式重排雷局的最大尝试次数（用于拒绝采样）
const RELAYOUT_MAX_TRIALS: usize = 20;

/// 没有时间、像素观念的局面状态机，侧重分析操作与局面的交互、推衍局面。在线地统计左右双击次数、ce次数、左键、右键、双击、当前解决的3BV。  
/// - 局限：不关注具体的线路（没有像素观念），因此不能计算path等。  
/// - 注意：ce的计算与扫雷网是不同的，本工具箱中，重复标同一个雷只算一个ce，即反复标雷、取消标雷不算作ce。
/// 应用场景：强化学习训练AI、游戏复盘计算指标。不能处理高亮（18）、算法确定是雷（12）等标记。  
/// - 用python调用时的示例：
/// ```python
/// import ms_toollib as ms
/// board = [
///     [0, 0, 1, -1, 2, 1, 1, -1],
///     [0, 0, 2, 3, -1, 3, 3, 2],
///     [1, 1, 3, -1, 4, -1, -1, 2],
///     [2, -1, 4, -1, 3, 4, -1, 4],
///     [3, -1, 5, 2, 1, 3, -1, -1],
///     [3, -1, -1, 2, 1, 2, -1, 3],
///     [-1, 5, 4, -1, 1, 1, 2, 2],
///     [-1, 3, -1, 2, 1, 0, 1, -1],
///     ];
/// v = ms.MinesweeperBoard(board) # 实例化后再用
/// v.step('lc', (0, 0)) # 左键按下
/// v.step('lr', (0, 0)) # 左键弹起
/// print('左键次数: ', v.left)
/// print('右键次数: ', v.right)
/// print('ce数: ', v.ces)
/// print('标雷数: ', v.flag)
/// print('解决3BV数: ', v.solved3BV)
/// print('局面: ', v.game_board)
/// ```
#[derive(Clone)]
pub struct MinesweeperBoard<T> {
    pub board: T,
    /// 局面
    pub game_board: Vec<Vec<i32>>,
    flaged_list: Vec<(usize, usize)>, // 记录哪些雷曾经被标过，则再标这些雷不记为ce
    /// 左键数
    pub left: usize,
    /// 右键数
    pub right: usize,
    /// 双击数
    pub double: usize,
    /// 左键ce数，ce = lce + rce + dce
    pub lce: usize,
    /// 右键ce数，ce = lce + rce + dce
    pub rce: usize,
    /// 双键ce数，ce = lce + rce + dce
    pub dce: usize,
    /// 标雷数
    pub flag: usize,
    /// 已解决的3BV数
    pub bbbv_solved: usize,
    pub row: usize,
    pub column: usize,
    pub mouse_state: MouseState,
    pub game_board_state: GameBoardState,
    // 指针，用于判断局面是否全部扫开
    pointer_x: usize,
    pointer_y: usize,
    pre_flag_num: usize,
    // 中键是否按下，配合“m”、“mc”、“mr”。
    middle_hold: bool,
    // 是否曾经修改过局面，若修改过，结算时要从头算，因为ce没法增量算
    pub board_changed: bool,
    /// 游戏模式，采用 evf 标准模式码。0 标准、6 强无猜、9 强可猜、10 弱可猜、
    /// 11–16 递归系列。其余模式状态机按标准规则处理。
    pub mode: u16,
    /// 是否开启问号标记。开启后右键循环为 10→11→17→10；关闭为 10→11→10。
    /// 问号格不参与双击（既不是雷也不算未打开的阻碍），但会阻挡获胜判定。
    pub use_question: bool,
}

impl Default for MinesweeperBoard<Vec<Vec<i32>>> {
    fn default() -> Self {
        MinesweeperBoard {
            board: vec![],
            game_board: vec![],
            flaged_list: vec![],
            left: 0,
            right: 0,
            double: 0,
            lce: 0,
            rce: 0,
            dce: 0,
            flag: 0,
            bbbv_solved: 0,
            row: 0,
            column: 0,
            mouse_state: MouseState::Undefined,
            game_board_state: GameBoardState::Ready,
            pointer_x: 0,
            pointer_y: 0,
            pre_flag_num: 0,
            middle_hold: false,
            board_changed: false,
            mode: 0,
            use_question: false,
        }
    }
}

#[cfg(any(feature = "py", feature = "rs"))]
impl Default for MinesweeperBoard<SafeBoard> {
    fn default() -> Self {
        MinesweeperBoard {
            board: SafeBoard::new(vec![]),
            game_board: vec![],
            flaged_list: vec![],
            left: 0,
            right: 0,
            double: 0,
            lce: 0,
            rce: 0,
            dce: 0,
            flag: 0,
            bbbv_solved: 0,
            row: 0,
            column: 0,
            mouse_state: MouseState::Undefined,
            game_board_state: GameBoardState::Ready,
            pointer_x: 0,
            pointer_y: 0,
            pre_flag_num: 0,
            middle_hold: false,
            board_changed: false,
            mode: 0,
            use_question: false,
        }
    }
}

impl MinesweeperBoard<Vec<Vec<i32>>> {
    pub fn new(board: Vec<Vec<i32>>) -> MinesweeperBoard<Vec<Vec<i32>>> {
        let row = board.get_row();
        let column = board.get_column();
        MinesweeperBoard {
            board,
            row,
            column,
            game_board: vec![vec![10; column]; row],
            flaged_list: vec![],
            mouse_state: MouseState::UpUp,
            ..MinesweeperBoard::<Vec<Vec<i32>>>::default()
        }
    }
    // pub fn set_board(&mut self, board: Vec<Vec<i32>>) {
    //     self.board = board;
    // }
}

#[cfg(any(feature = "py", feature = "rs"))]
impl MinesweeperBoard<SafeBoard> {
    pub fn new(board: SafeBoard) -> MinesweeperBoard<SafeBoard> {
        let row = board.get_row();
        let column = board.get_column();
        MinesweeperBoard {
            board,
            row,
            column,
            game_board: vec![vec![10; column]; row],
            flaged_list: vec![],
            mouse_state: MouseState::UpUp,
            ..MinesweeperBoard::<SafeBoard>::default()
        }
    }
}

impl<T: EmptyBoard> MinesweeperBoard<T> {
    pub fn new_from_board(row: usize, column: usize) -> Self {
        MinesweeperBoard {
            board: T::empty_board(row, column),
            game_board: vec![vec![10; column]; row],
            flaged_list: vec![],
            left: 0,
            right: 0,
            double: 0,
            lce: 0,
            rce: 0,
            dce: 0,
            flag: 0,
            bbbv_solved: 0,
            row,
            column,
            mouse_state: MouseState::UpUp,
            game_board_state: GameBoardState::Ready,
            pointer_x: 0,
            pointer_y: 0,
            pre_flag_num: 0,
            middle_hold: false,
            board_changed: false,
            mode: 0,
            use_question: false,
        }
    }
}

impl<T> MinesweeperBoard<T> {
    // 初始化。对应强化学习领域gym的api中的reset。
    pub fn reset(&mut self) {
        self.game_board = vec![vec![10; self.column]; self.row];
        // self.board = self.board.new(self.row, self.column);
        self.left = 0;
        self.right = 0;
        self.double = 0;
        self.lce = 0;
        self.rce = 0;
        self.dce = 0;
        self.flag = 0;
        self.left = 0;
        self.bbbv_solved = 0;
        self.flaged_list = vec![];
        self.mouse_state = MouseState::UpUp;
        if self.game_board_state != GameBoardState::Display {
            self.game_board_state = GameBoardState::Ready;
        }
        self.pointer_x = 0;
        self.pointer_y = 0;
        self.pre_flag_num = 0;
        self.middle_hold = false;
        self.board_changed = false;
    }
    /// 可猜模式改局面
    pub fn set_board(&mut self, board: T) {
        self.board = board;
        self.pointer_x = 0;
        self.pointer_y = 0;
        self.board_changed = true;
    }
    /// 设置游戏模式（evf 标准模式码）
    pub fn set_mode(&mut self, mode: u16) {
        self.mode = mode;
    }
    pub fn replace_board(&mut self, board: T) {
        self.board = board;
        self.pointer_x = 0;
        self.pointer_y = 0;
        self.board_changed = true;
    }
    /// 模式是否要求重排后的雷局仍然可解
    fn mode_requires_solvable(&self) -> bool {
        matches!(self.mode, 9 | 14 | 15 | 16)
    }
    /// 模式是否属于可猜系列（9/10）。14/15/16 是"无猜 X 递归"，不叠加可猜重排。
    fn mode_is_guess(&self) -> bool {
        matches!(self.mode, 9 | 10)
    }
    /// 双击递归：手动双击后自动扫全局执行（11 纯双击递归、13/16 双击标雷递归、14 无猜双击递归）
    fn mode_auto_chord(&self) -> bool {
        matches!(self.mode, 11 | 13 | 14 | 16)
    }
    /// 标雷递归：手动标雷后自动扫全局执行（12 纯标雷递归、13/16 双击标雷递归、15 无猜标雷递归）
    fn mode_auto_flag(&self) -> bool {
        matches!(self.mode, 12 | 13 | 15 | 16)
    }
    /// 双击标雷递归：交替执行标雷→双击直到都不能动（13、16）
    fn mode_both_recursive(&self) -> bool {
        matches!(self.mode, 13 | 16)
    }
    /// 尝试重排雷局。找不到满足条件的雷局时返回 `None`。
    fn try_relayout(
        &self,
        must_not_mine: &[(usize, usize)],
        must_mine: &[(usize, usize)],
        require_solvable: bool,
    ) -> Option<Vec<Vec<i32>>>
    where
        T: ReplaceBoard,
    {
        let board = self.board.to_vec_vec();
        relayout::relayout(
            &board,
            &self.game_board,
            must_not_mine,
            must_mine,
            require_solvable,
            RELAYOUT_MAX_TRIALS,
        )
    }
    /// 用新的雷局替换当前雷局，并重置胜利扫描游标。
    fn replace_board_vec(&mut self, board: Vec<Vec<i32>>)
    where
        T: ReplaceBoard,
    {
        self.board.replace_with(board);
        self.pointer_x = 0;
        self.pointer_y = 0;
        self.board_changed = true;
    }
    /// 当前局面是否存在可以用逻辑证明一定不是雷的未打开格子。
    /// 弱无猜（7）只有在“还有可推出的安全格”时才判负；一个都推不出来（必须猜）时豁免。
    fn has_provable_safe_cell(&self) -> bool
    where
        T: ReplaceBoard,
    {
        relayout::has_provable_safe_cell(&self.game_board)
    }
    /// Playing状态下的左击，没有按下抬起之分
    fn left_click(&mut self, x: usize, y: usize, is_opening: bool) -> Result<u8, ()>
    where
        T: std::ops::Index<usize> + BoardSize + ReplaceBoard,
        T::Output: std::ops::Index<usize, Output = i32>,
    {
        self.left += 1;
        if self.game_board[x][y] != 10 && self.game_board[x][y] != 12 {
            return Ok(0);
        }
        // 开局第一击豁免模式判定
        if !is_opening && self.game_board[x][y] == 10 {
            if self.mode == 6 {
                // 强无猜：只要该格“可能是雷”，就重排成该格为雷的雷局，然后按踩雷判负
                if self.board[x][y] != -1 {
                    if let Some(new_board) = self.try_relayout(&[], &[(x, y)], false) {
                        self.replace_board_vec(new_board);
                    }
                }
            } else if self.mode == 7 {
                // 弱无猜：局面里还有能用逻辑证明的安全格时，点到“可能是雷”的格子即判负；
                // 一个能证明的安全格都没有（只能猜）时，这一击无论点在哪都不判负
                if self.board[x][y] == -1 {
                    if !self.has_provable_safe_cell() {
                        if let Some(new_board) = self.try_relayout(&[(x, y)], &[], false) {
                            self.replace_board_vec(new_board);
                        }
                    }
                } else if self.has_provable_safe_cell() {
                    // 该格“可能是雷”：存在一致雷局让它为雷，就把雷局换成其中之一，判负
                    if let Some(new_board) = self.try_relayout(&[], &[(x, y)], false) {
                        self.replace_board_vec(new_board);
                    }
                }
            } else if self.mode == 8 {
                // 准无猜：只有推不出任何安全格（只能猜）时，点到真雷才重排成非雷
                if self.board[x][y] == -1 && !self.has_provable_safe_cell() {
                    if let Some(new_board) = self.try_relayout(&[(x, y)], &[], false) {
                        self.replace_board_vec(new_board);
                    }
                }
            } else if self.mode_is_guess() && self.board[x][y] == -1 {
                // 可猜：点到的实际是雷时，重排成该格非雷（9/14/15/16 还要求可解）
                let require_solvable = self.mode_requires_solvable();
                if let Some(new_board) =
                    self.try_relayout(&[(x, y)], &[], require_solvable)
                {
                    self.replace_board_vec(new_board);
                }
            }
        }
        // refresh_board(&self.board, &mut self.game_board, vec![(x, y)]);
        match self.board[x][y] {
            0 => {
                if self.cell_is_op_completed(x, y, &mut vec![vec![false; self.column]; self.row]) {
                    self.bbbv_solved += 1;
                }
                self.lce += 1;
                refresh_board(&self.board, &mut self.game_board, vec![(x, y)]);
                if self.is_win() {
                    self.game_board_state = GameBoardState::Win;
                }
                Ok(2)
            }
            -1 => {
                refresh_board(&self.board, &mut self.game_board, vec![(x, y)]);
                self.game_board_state = GameBoardState::Loss;
                Ok(4)
            }
            _ => {
                refresh_board(&self.board, &mut self.game_board, vec![(x, y)]);
                if self.cell_is_bbbv(x, y) {
                    self.bbbv_solved += 1;
                }
                self.lce += 1;
                if self.is_win() {
                    self.game_board_state = GameBoardState::Win;
                }
                Ok(2)
            }
        }
    }
    /// Playing状态下的右击，没有按下抬起之分
    fn right_click(&mut self, x: usize, y: usize) -> Result<u8, ()>
    where
        T: std::ops::Index<usize> + BoardSize + std::fmt::Debug + ReplaceBoard,
        T::Output: std::ops::Index<usize, Output = i32>,
    {
        self.right += 1;
        if self.game_board[x][y] < 10 {
            return Ok(0);
        } else {
            // println!("{:?}", self.board);
            let mut flag_added = false;
            if self.board[x][y] != -1 {
                match self.game_board[x][y] {
                    10 => {
                        self.game_board[x][y] = 11;
                        self.flag += 1;
                        flag_added = true;
                    }
                    11 => {
                        // 开问号时先退为问号，再右击才取消标记
                        self.game_board[x][y] = if self.use_question { 17 } else { 10 };
                        self.flag -= 1;
                    }
                    17 => {
                        self.game_board[x][y] = 10;
                    }
                    _ => return Err(()),
                }
            } else {
                match self.game_board[x][y] {
                    10 => {
                        self.game_board[x][y] = 11;
                        self.flag += 1;
                        if !self.flaged_list.contains(&(x, y)) {
                            self.rce += 1;
                        }
                        self.flaged_list.push((x, y));
                        flag_added = true;
                    }
                    11 => {
                        self.game_board[x][y] = if self.use_question { 17 } else { 10 };
                        self.flag -= 1;
                    }
                    17 => {
                        self.game_board[x][y] = 10;
                    }
                    _ => return Err(()),
                }
            }
            // 标雷递归 / 双击标雷递归：手动标雷后自动跑递归
            if flag_added
                && self.game_board_state == GameBoardState::Playing
                && self.mode_auto_flag()
            {
                if self.mode_both_recursive() {
                    self.run_both_recursive();
                } else {
                    self.run_flag_recursive();
                }
            }
            Ok(1)
        }
    }
    /// Playing状态下的双击，没有按下抬起之分
    fn chording_click(&mut self, x: usize, y: usize) -> Result<u8, ()>
    where
        T: std::ops::Index<usize> + BoardSize + ReplaceBoard,
        T::Output: std::ops::Index<usize, Output = i32>,
    {
        self.double += 1;
        let result = self.perform_chord(x, y, false)?;
        if self.game_board_state == GameBoardState::Loss {
            return Ok(4);
        }
        if matches!(self.game_board_state, GameBoardState::Playing) && self.mode_auto_chord() {
            if self.mode_both_recursive() {
                self.run_both_recursive();
            } else {
                self.run_chord_recursive();
            }
        }
        Ok(result)
    }
    /// 执行双击的实质部分。`silent` 为 true 时是递归产生的自动双击，
    /// 不计玩家操作次数（double/dce），但仍计入 bbbv_solved 并正常判胜负。
    fn perform_chord(&mut self, x: usize, y: usize, silent: bool) -> Result<u8, ()>
    where
        T: std::ops::Index<usize> + BoardSize + ReplaceBoard,
        T::Output: std::ops::Index<usize, Output = i32>,
    {
        // 在8、10、11、12上双击一定不会发生任何事情，但是在0上双击仍然可能有效！
        if self.game_board[x][y] >= 8 {
            return Ok(0);
        }
        let mut flag_chording_useful = false; // 双击有效的基础上，周围是否有未打开的格子
        let mut chording_cells = vec![]; // 未打开的格子的集合
        let mut flaged_num = 0; // 双击点周围的标雷数
        for i in max(1, x) - 1..min(self.row, x + 2) {
            for j in max(1, y) - 1..min(self.column, y + 2) {
                if i != x || j != y {
                    if self.game_board[i][j] == 11 {
                        flaged_num += 1
                    }
                    if self.game_board[i][j] == 10 {
                        chording_cells.push((i, j));
                        flag_chording_useful = true;
                    }
                }
            }
        }
        if flaged_num != self.game_board[x][y] || !flag_chording_useful {
            return Ok(0);
        }

        // 模式处理：必要时重排雷局
        // 严苛判定：6 强无猜始终严苛；7 弱无猜只要局面里还有可推出的安全格就严苛
        let provable_safe = if matches!(self.mode, 7 | 8) {
            self.has_provable_safe_cell()
        } else {
            false
        };
        let strict = self.mode == 6 || (self.mode == 7 && provable_safe);
        if strict {
            // 强无猜：只要本次将打开的格子中存在“可能是雷”的格子，就重排成它为雷的雷局
            if !chording_cells
                .iter()
                .any(|c| self.board[c.0][c.1] == -1)
            {
                let mut replacement = None;
                for &cell in &chording_cells {
                    if let Some(new_board) = self.try_relayout(&[], &[cell], false) {
                        replacement = Some(new_board);
                        break;
                    }
                }
                if let Some(new_board) = replacement {
                    self.replace_board_vec(new_board);
                }
            }
        } else if (matches!(self.mode, 7 | 8) && !provable_safe) || self.mode_is_guess() {
            // 可猜、准无猜、无解局面下的弱无猜：本次将打开的实际是雷的格子，重排为非雷
            let mines: Vec<(usize, usize)> = chording_cells
                .iter()
                .filter(|c| self.board[c.0][c.1] == -1)
                .cloned()
                .collect();
            if !mines.is_empty() {
                let require_solvable = self.mode_requires_solvable();
                // 弱无猜/准无猜保证玩家不会被判负，优先让所有将被打开的格子都非雷；
                // 做不到时退而只保证原本是雷的格子非雷
                let mut new_board = None;
                if matches!(self.mode, 7 | 8) {
                    new_board = self.try_relayout(&chording_cells, &[], require_solvable);
                }
                if new_board.is_none() {
                    new_board = self.try_relayout(&mines, &[], require_solvable);
                }
                if let Some(new_board) = new_board {
                    self.replace_board_vec(new_board);
                }
            }
        }

        let mut surround_bbbv = 0; // 周围的3BV
        for &(i, j) in &chording_cells {
            if self.board[i][j] > 0 && self.cell_is_bbbv(i, j) {
                surround_bbbv += 1;
            }
        }
        // 先判断是否踩雷，踩雷时不能再走获胜判定，否则可能把 Loss 覆盖成 Win
        let hit_mine = chording_cells
            .iter()
            .any(|ch| self.board[ch.0][ch.1] == -1);
        if !silent {
            self.dce += 1;
        }
        self.bbbv_solved += surround_bbbv;
        self.bbbv_solved += self.op_num_around_cell(x, y);
        refresh_board(&self.board, &mut self.game_board, chording_cells);

        if hit_mine {
            self.game_board_state = GameBoardState::Loss;
            return Ok(4);
        }
        if self.is_win() {
            self.game_board_state = GameBoardState::Win;
        }
        Ok(3)
    }
    /// 判断某个已打开的数字格是否可以（安全地）双击
    fn chord_is_useful(&self, x: usize, y: usize) -> bool
    where
        T: std::ops::Index<usize> + BoardSize,
        T::Output: std::ops::Index<usize, Output = i32>,
    {
        if self.game_board[x][y] >= 8 || self.game_board[x][y] < 0 {
            return false;
        }
        let mut flaged_num = 0;
        let mut unopened = false;
        for i in max(1, x) - 1..min(self.row, x + 2) {
            for j in max(1, y) - 1..min(self.column, y + 2) {
                if self.game_board[i][j] == 11 {
                    flaged_num += 1;
                } else if self.game_board[i][j] == 10 {
                    unopened = true;
                }
            }
        }
        flaged_num == self.game_board[x][y] && unopened
    }
    /// 单集合逻辑找必标雷：对每个已打开数字格，若 (数字 - 周围已标雷数) == 周围未开格数，
    /// 则所有未开格都是雷。不使用子集差、枚举等高阶计算。
    /// 返回本轮找到的新雷坐标列表（可能已被标过的不会重复返回）。
    fn simple_flag_seek(&self) -> Vec<(usize, usize)>
    where
        T: std::ops::Index<usize> + BoardSize,
        T::Output: std::ops::Index<usize, Output = i32>,
    {
        let mut result = Vec::new();
        for i in 0..self.row {
            for j in 0..self.column {
                let v = self.game_board[i][j];
                if v <= 0 || v >= 9 {
                    continue; // 只对已打开的 1-8 数字生效
                }
                let mut flagged = 0i32;
                let mut unopened: Vec<(usize, usize)> = Vec::new();
                for ni in i.saturating_sub(1)..=i.saturating_add(1).min(self.row - 1) {
                    for nj in j.saturating_sub(1)..=j.saturating_add(1).min(self.column - 1) {
                        if ni == i && nj == j { continue; }
                        match self.game_board[ni][nj] {
                            11 => flagged += 1,
                            10 => unopened.push((ni, nj)),
                            _ => {}
                        }
                    }
                }
                let remaining = v as i32 - flagged;
                if remaining > 0 && remaining == unopened.len() as i32 {
                    for pos in unopened {
                        // 可能已被前一个数字格标过，跳过
                        if !result.contains(&pos) && self.game_board[pos.0][pos.1] == 10 {
                            result.push(pos);
                        }
                    }
                }
            }
        }
        result
    }
    /// 自动标雷：反复跑 simple_flag_seek，直到找不到新雷。
    fn run_flag_recursive(&mut self)
    where
        T: std::ops::Index<usize> + BoardSize + ReplaceBoard,
        T::Output: std::ops::Index<usize, Output = i32>,
    {
        loop {
            if matches!(self.game_board_state, GameBoardState::Win | GameBoardState::Loss) { break; }
            let mines = self.simple_flag_seek();
            if mines.is_empty() { break; }
            for (x, y) in mines {
                if !matches!(self.game_board_state, GameBoardState::Playing) { break; }
                if self.game_board[x][y] == 10 {
                    self.game_board[x][y] = 11;
                    self.flag += 1;
                    if self.board[x][y] == -1 && !self.flaged_list.contains(&(x, y)) {
                        self.flaged_list.push((x, y));
                    }
                }
            }
        }
    }
    /// 自动双击：反复扫全局找所有 chord_is_useful 的格子静默双击，直到没有新格被打开。
    fn run_chord_recursive(&mut self)
    where
        T: std::ops::Index<usize> + BoardSize + ReplaceBoard,
        T::Output: std::ops::Index<usize, Output = i32>,
    {
        loop {
            if matches!(self.game_board_state, GameBoardState::Win | GameBoardState::Loss) { break; }
            let mut any = false;
            for i in 0..self.row {
                for j in 0..self.column {
                    if !matches!(self.game_board_state, GameBoardState::Playing) { break; }
                    if self.chord_is_useful(i, j) {
                        let before = self.game_board.clone();
                        let _ = self.perform_chord(i, j, true);
                        if self.game_board_state == GameBoardState::Loss { return; }
                        if !newly_opened(&before, &self.game_board).is_empty() {
                            any = true;
                        }
                    }
                }
            }
            if !any { break; }
        }
    }
    /// 双击标雷递归：交替跑标雷→双击→标雷→双击，直到两者都无新动作。
    fn run_both_recursive(&mut self)
    where
        T: std::ops::Index<usize> + BoardSize + ReplaceBoard,
        T::Output: std::ops::Index<usize, Output = i32>,
    {
        loop {
            if matches!(self.game_board_state, GameBoardState::Win | GameBoardState::Loss) { break; }
            self.run_flag_recursive();
            if !matches!(self.game_board_state, GameBoardState::Playing) { break; }
            let before = self.game_board.clone();
            self.run_chord_recursive();
            if self.game_board_state == GameBoardState::Loss { return; }
            if self.game_board == before { break; }
        }
    }
    fn cell_is_bbbv(&self, x: usize, y: usize) -> bool
    where
        T: std::ops::Index<usize> + BoardSize,
        T::Output: std::ops::Index<usize, Output = i32>,
    {
        // 判断该大于0的数字是不是3BV
        // 如果是0，即使是3bv，依然返回false
        if self.board[x][y] <= 0 {
            return false;
        }
        for i in max(1, x) - 1..min(self.row, x + 2) {
            for j in max(1, y) - 1..min(self.column, y + 2) {
                if self.board[i][j] == 0 {
                    return false;
                }
            }
        }
        true
    }
    // 在传入格子上双击以后，将新打开的（完整）op数。（已打开的不算）
    // 这个格子不是雷、双击是合法的
    // 只可能返回0，1，2
    fn op_num_around_cell(&self, x: usize, y: usize) -> usize
    where
        T: std::ops::Index<usize> + BoardSize,
        T::Output: std::ops::Index<usize, Output = i32>,
    {
        let mut op_num = 0;
        let mut game_board_mark = vec![vec![false; self.column]; self.row];
        for i in max(1, x) - 1..min(self.row, x + 2) {
            for j in max(1, y) - 1..min(self.column, y + 2) {
                if self.game_board[i][j] == 10 && self.board[i][j] == 0 && !game_board_mark[i][j] {
                    if self.cell_is_op_completed(i, j, &mut game_board_mark) {
                        op_num += 1;
                    }
                }
            }
        }
        op_num
    }

    // 是局面上能够完全打开的op（在游戏局面上是没有打开的状态，点下后能够完全打开）
    // 这是为了防范一种特殊情况，在空上面右击标雷使得空打开的时候不能完全打开；没有完全打开的空，不应计入3bv。
    fn cell_is_op_completed(&self, x: usize, y: usize, game_board_mark: &mut Vec<Vec<bool>>) -> bool
    where
        T: std::ops::Index<usize> + BoardSize,
        T::Output: std::ops::Index<usize, Output = i32>,
    {
        // let mut game_board_mark = vec![vec![false; self.column]; self.row];
        let mut poses = vec![(x, y)];
        while let Some(top) = poses.pop() {
            let (i, j) = top;
            if self.board[i][j] > 0 {
                continue;
            } else if self.board[i][j] == 0 {
                if self.game_board[i][j] == 11 {
                    return false;
                }
                game_board_mark[i][j] = true;
                for m in max(1, i) - 1..min(self.row, i + 2) {
                    for n in max(1, j) - 1..min(self.column, j + 2) {
                        if (i != m || j != n) && (!game_board_mark[m][n]) {
                            poses.push((m, n));
                        }
                    }
                }
            } else {
            }
        }
        true
    }
    /// 返回的值的含义是：0：没有任何作用的操作，例如左键数字、踩雷。  
    /// 1：推进了局面，但没有改变ai对局面的判断，特指标雷。  
    /// 2：改变局面的操作，左键、双击。
    /// 3: 正确的双击.   
    /// e的类型有11种，lc（左键按下）, lr（左键抬起）, rc（右键按下）, rr（右键抬起）, mc（中键按下）, mr（中键抬起）,   
    ///     cc（双键按下，但不确定是哪个键），pf（在开始前预先标雷，而又失去了标记的过程）,   
    ///     l（左键按下或抬起）, r（右键按下或抬起）, m（中键按下或抬起）。  
    /// ## 注意事项：
    /// - 在理想的鼠标状态机中，有些情况是不可能的，例如右键没有抬起就按下两次，但在阿比特中就观察到这种事情。
    // 无法正确处理局面外按下的事件（lc、rc、mc等），因为局面外按下在局面内释放，
    // 和局面内按下在局面内释放不一样，前者的释放不起作用，而后者的释放会触发点击。因此，同样是
    // 左键按下，不能简单用MouseState::DownUp来描述，目前的状态机状态数量不够。
    // 因此必须在UI层面防止调用，否则会产生意料之外的效果。
    pub fn step(&mut self, e: &str, pos: (usize, usize)) -> Result<u8, ()>
    where
        T: std::ops::Index<usize> + BoardSize + std::fmt::Debug + ReplaceBoard,
        T::Output: std::ops::Index<usize, Output = i32>,
    {
        assert!(
            (pos.0 < self.row && pos.1 < self.column)
                || (pos.0 == self.row && pos.1 == self.column),
            "pos out of range"
        );
        // "l"/"r"/"m" 是不确定按下还是抬起的事件（evf 标准），先按鼠标状态机解析成确定事件。
        // 左键按下的状态：DownUp、Chording、ChordingNotFlag、DownUpAfterChording；
        // 右键按下的状态：UpDown、Chording、ChordingNotFlag；
        // 中键由 middle_hold 记录：按下翻译成 "mc"，抬起翻译成 "mr"。
        let e = match e {
            "l" => match self.mouse_state {
                MouseState::DownUp
                | MouseState::Chording
                | MouseState::ChordingNotFlag
                | MouseState::DownUpAfterChording => "lr",
                _ => "lc",
            },
            "r" => match self.mouse_state {
                MouseState::UpDown | MouseState::Chording | MouseState::ChordingNotFlag => "rr",
                _ => "rc",
            },
            "m" => {
                self.middle_hold = !self.middle_hold;
                if self.middle_hold {
                    "mc"
                } else {
                    "mr"
                }
            }
            other => other,
        };
        match self.game_board_state {
            GameBoardState::Ready => match e {
                "mv" | "mc" | "mr" => {
                    // arbiter这个奇葩，录像可能以中键的按下和抬起开始
                    return Ok(0);
                }
                "lc" => {
                    //  "l"处理不了，很复杂，以后再说
                    match self.mouse_state {
                        MouseState::UpUp => {
                            self.mouse_state = MouseState::DownUp;
                            if pos.0 < self.row && pos.1 < self.column {
                                self.game_board_state = GameBoardState::PreFlaging;
                            }
                        }
                        MouseState::UpDown => self.mouse_state = MouseState::Chording,
                        MouseState::UpDownNotFlag => self.mouse_state = MouseState::ChordingNotFlag,

                        // lc(pf)->rc->失焦->lc
                        // lc(pf)->失焦->rc(pf,Chording)->rr(rdy,DownUpAfterChording)->lc
                        _ => {}
                    }
                    return Ok(0);
                }
                "pf" => {
                    assert!(self.game_board[pos.0][pos.1] == 10, "");
                    self.pre_flag_num += 1;
                    self.game_board_state = GameBoardState::PreFlaging;
                    return self.right_click(pos.0, pos.1);
                }
                //  "r"处理不了，很复杂，以后再说
                "rc" => match self.mouse_state {
                    MouseState::UpUp => {
                        self.mouse_state = MouseState::UpDown;
                        if pos.0 < self.row && pos.1 < self.column {
                            self.pre_flag_num = 1;
                            self.game_board_state = GameBoardState::PreFlaging;
                            return self.right_click(pos.0, pos.1);
                        }
                        return Ok(0);
                    }
                    MouseState::DownUpAfterChording => {
                        self.mouse_state = MouseState::Chording;
                        return Ok(0);
                    }
                    // _ => return Err(()),
                    _ => {}
                },
                "lr" => match self.mouse_state {
                    MouseState::Chording | MouseState::ChordingNotFlag => {
                        self.mouse_state = MouseState::UpDown;
                        return Ok(0);
                    }
                    MouseState::DownUpAfterChording => {
                        self.mouse_state = MouseState::UpUp;
                        return Ok(0);
                    }
                    MouseState::DownUp | MouseState::UpUp => {
                        // 按理进不来
                        // 局面外按下鼠标（所以没有进入preflag状态），再在局面内松开鼠标时，
                        // 元扫雷的ui框架都不会有鼠标事件，但是其他扫雷可能有，并且鼠标释放起作用
                        self.mouse_state = MouseState::UpUp;
                        return Ok(0);
                    }
                    // _ => return Err(()),
                    _ => {}
                },
                "rr" => match self.mouse_state {
                    MouseState::UpDown => {
                        self.mouse_state = MouseState::UpUp;
                        return Ok(0);
                    }
                    MouseState::Chording => {
                        self.mouse_state = MouseState::DownUpAfterChording;
                        return Ok(0);
                    }
                    MouseState::UpUp => {
                        // 双键按下时按快捷键重开
                        return Ok(0);
                    }
                    // _ => return Err(()),
                    _ => {}
                },
                "cc" => {
                    match self.mouse_state {
                        MouseState::DownUp => self.mouse_state = MouseState::Chording,
                        MouseState::DownUpAfterChording => self.mouse_state = MouseState::Chording,
                        MouseState::UpDown => self.mouse_state = MouseState::Chording,
                        MouseState::UpDownNotFlag => self.mouse_state = MouseState::ChordingNotFlag,
                        MouseState::UpUp => {
                            // 单键按下时按快捷键重开，再双键
                            self.mouse_state = MouseState::Chording;
                            return Ok(0);
                        }
                        // _ => return Err(()),
                        _ => {}
                    }
                    return Ok(0);
                }
                _ => return Err(()),
            },
            GameBoardState::PreFlaging => match e {
                "lc" | "rr" | "mv" => {} // 和playing状态下恰好一致，主要是计算点击次数
                "lr" => match self.mouse_state {
                    MouseState::DownUp => {
                        if pos.0 == self.row && pos.1 == self.column {
                            self.mouse_state = MouseState::UpUp;
                            if self.pre_flag_num == 0 {
                                self.game_board_state = GameBoardState::Ready;
                                self.clear_click_num();
                            } else {
                                // 预标雷阶段，在局面外左键弹起
                                self.left += 1;
                            }
                            return Ok(0);
                        }
                        if self.game_board[pos.0][pos.1] == 10 {
                            // 预标雷阶段，在10上左键弹起
                            self.game_board_state = GameBoardState::Playing;
                            self.mouse_state = MouseState::UpUp;
                            return self.left_click(pos.0, pos.1, true);
                        } else {
                            // 预标雷阶段，在旗上左键弹起。左键已抬起，必须恢复鼠标状态，
                            // 否则后续右击会被当成双击的右键跟进，导致旗再也退不掉
                            self.left += 1;
                            self.mouse_state = MouseState::UpUp;
                            return Ok(0);
                        }
                    }
                    // _ => return Err(()),
                    _ => {}
                },
                "pf" => {
                    assert!(self.game_board[pos.0][pos.1] == 10, "");
                    self.pre_flag_num += 1;
                    return self.right_click(pos.0, pos.1);
                }
                "rc" => match self.mouse_state {
                    MouseState::UpUp => {
                        if pos.0 < self.row && pos.1 < self.column {
                            self.mouse_state = MouseState::UpDown;
                            // 预标雷阶段，要么10，要么11（开问号时还有17），不可能出现<10
                            match self.game_board[pos.0][pos.1] {
                                10 => {
                                    self.pre_flag_num += 1;
                                    self.game_board_state = GameBoardState::PreFlaging;
                                }
                                11 => {
                                    self.pre_flag_num -= 1;
                                    if !self.use_question && self.pre_flag_num == 0 {
                                        self.game_board_state = GameBoardState::Ready;
                                        self.clear_click_num();
                                        self.game_board[pos.0][pos.1] = 10;
                                        return Ok(0);
                                    }
                                    // 开问号时 11→17，局面上仍留问号标记，保持 PreFlaging
                                }
                                17 => {
                                    // 17→10，pre_flag_num 不变；没有剩余旗时回到 Ready
                                    let r = self.right_click(pos.0, pos.1);
                                    if self.pre_flag_num == 0 {
                                        self.game_board_state = GameBoardState::Ready;
                                        self.clear_click_num();
                                    }
                                    return r;
                                }
                                _ => return Err(()),
                            }
                            return self.right_click(pos.0, pos.1);
                        } else {
                            self.mouse_state = MouseState::UpDownNotFlag;
                            self.right += 1;
                        }
                    }
                    MouseState::DownUp => {
                        self.mouse_state = MouseState::Chording;
                        if pos.0 < self.row && pos.1 < self.column {
                            if self.pre_flag_num == 0 {
                                self.game_board_state = GameBoardState::Ready;
                            }
                        }
                    }
                    MouseState::DownUpAfterChording => self.mouse_state = MouseState::Chording,
                    _ => return Err(()),
                },
                "cc" => {
                    match self.mouse_state {
                        MouseState::DownUp => {
                            if pos.0 < self.row && pos.1 < self.column {
                                if self.pre_flag_num == 0 {
                                    self.game_board_state = GameBoardState::Ready;
                                }
                            }
                            self.mouse_state = MouseState::Chording;
                        }
                        MouseState::DownUpAfterChording => self.mouse_state = MouseState::Chording,
                        MouseState::UpDown => self.mouse_state = MouseState::Chording,
                        MouseState::UpDownNotFlag => self.mouse_state = MouseState::ChordingNotFlag,
                        MouseState::UpUp => self.mouse_state = MouseState::Chording,
                        _ => return Err(()),
                    }
                    return Ok(0);
                }
                "mc" => {
                    // 阿比特中有在预标雷阶段中键按下弹起
                    self.middle_hold = true;
                    return Ok(0);
                }
                "mr" => {
                    self.middle_hold = false;
                    return Ok(0);
                }
                _ => return Err(()),
            },
            GameBoardState::Playing => {}
            _ => return Ok(0),
        }
        // 状态为playing，就继续往下走
        match e {
            "lc" => match self.mouse_state {
                MouseState::UpUp => self.mouse_state = MouseState::DownUp,
                MouseState::UpDown => self.mouse_state = MouseState::Chording,
                MouseState::UpDownNotFlag => self.mouse_state = MouseState::ChordingNotFlag,
                // 以下情况其实是不可能的
                MouseState::DownUp => {} // 左键按下后、窗口失焦、左键按下
                MouseState::DownUpAfterChording => {}
                MouseState::Chording => {}
                MouseState::ChordingNotFlag => {}
                MouseState::Undefined => self.mouse_state = MouseState::DownUp,
            },
            "lr" => match self.mouse_state {
                MouseState::DownUp => {
                    self.mouse_state = MouseState::UpUp;
                    if pos.0 == self.row && pos.1 == self.column {
                        // 局面外的左键也+1
                        self.left += 1;
                        return Ok(0);
                    }
                    // println!("x={:?}, y={:?}", pos.0, pos.1);
                    return self.left_click(pos.0, pos.1, false);
                }
                MouseState::Chording => {
                    self.mouse_state = MouseState::UpDown;
                    if pos.0 == self.row && pos.1 == self.column {
                        return Ok(0);
                    }
                    return self.chording_click(pos.0, pos.1);
                }
                MouseState::DownUpAfterChording => self.mouse_state = MouseState::UpUp,
                MouseState::ChordingNotFlag => {
                    self.mouse_state = MouseState::UpDown;
                    self.right -= 1;
                    if pos.0 == self.row && pos.1 == self.column {
                        return Ok(0);
                    }
                    return self.chording_click(pos.0, pos.1);
                }
                // 以下情况其实是不可能的
                MouseState::UpDown => {}
                MouseState::UpDownNotFlag => {}
                MouseState::UpUp => self.mouse_state = MouseState::UpUp,
                MouseState::Undefined => self.mouse_state = MouseState::UpUp,
            },
            "rc" => match self.mouse_state {
                MouseState::UpUp => {
                    // println!("{:?}", pos);
                    // 无论局面内外，playing时，右键按下，right都会+1
                    if pos.0 < self.row && pos.1 < self.column {
                        // println!("{:?}", self.game_board[pos.0][pos.1]);
                        if self.game_board[pos.0][pos.1] < 10 {
                            self.mouse_state = MouseState::UpDownNotFlag;
                        } else {
                            self.mouse_state = MouseState::UpDown;
                        }
                        return self.right_click(pos.0, pos.1);
                    } else {
                        self.mouse_state = MouseState::UpDownNotFlag;
                        self.right += 1;
                    }
                }
                MouseState::DownUp => self.mouse_state = MouseState::Chording,
                MouseState::DownUpAfterChording => self.mouse_state = MouseState::Chording,
                // 以下情况其实是不可能的
                MouseState::UpDown => {}
                MouseState::UpDownNotFlag => {}
                MouseState::Chording => {}
                MouseState::ChordingNotFlag => {}
                MouseState::Undefined => self.mouse_state = MouseState::UpDown,
            },
            "rr" => match self.mouse_state {
                MouseState::UpDown => self.mouse_state = MouseState::UpUp,
                MouseState::UpDownNotFlag => self.mouse_state = MouseState::UpUp,
                MouseState::Chording => {
                    self.mouse_state = MouseState::DownUpAfterChording;
                    if pos.0 == self.row && pos.1 == self.column {
                        return Ok(0);
                    }
                    return self.chording_click(pos.0, pos.1);
                }
                MouseState::ChordingNotFlag => {
                    self.mouse_state = MouseState::DownUpAfterChording;
                    self.right -= 1;
                    if pos.0 == self.row && pos.1 == self.column {
                        return Ok(0);
                    }
                    return self.chording_click(pos.0, pos.1);
                }
                // 以下情况其实是不可能的
                MouseState::DownUp => {}
                MouseState::DownUpAfterChording => {}
                MouseState::UpUp => self.mouse_state = MouseState::UpUp,
                MouseState::Undefined => self.mouse_state = MouseState::UpUp,
            },
            "mv" => {}
            "mc" => {
                self.middle_hold = true;
            }
            "mr" => {
                self.middle_hold = false;
                if pos.0 == self.row && pos.1 == self.column {
                    return Ok(0);
                }
                return self.chording_click(pos.0, pos.1);
            }
            "cc" => match self.mouse_state {
                MouseState::DownUp => self.mouse_state = MouseState::Chording,
                MouseState::DownUpAfterChording => self.mouse_state = MouseState::Chording,
                MouseState::UpDown => self.mouse_state = MouseState::Chording,
                MouseState::UpDownNotFlag => self.mouse_state = MouseState::ChordingNotFlag,
                MouseState::UpUp => self.mouse_state = MouseState::Chording,
                _ => return Err(()),
            },
            // "crl" => match self.mouse_state {
            //     MouseState::Chording => {
            //         self.mouse_state = MouseState::UpDown;
            //         if pos.0 == self.row && pos.1 == self.column {
            //             return Ok(0);
            //         }
            //         return self.chording_click(pos.0, pos.1);
            //     }
            //     MouseState::ChordingNotFlag => {
            //         self.mouse_state = MouseState::UpDown;
            //         self.right -= 1;
            //         if pos.0 == self.row && pos.1 == self.column {
            //             return Ok(0);
            //         }
            //         return self.chording_click(pos.0, pos.1);
            //     }
            //     _ => return Err(()),
            // },
            // "crr" => match self.mouse_state {
            //     MouseState::Chording => {
            //         self.mouse_state = MouseState::DownUpAfterChording;
            //         if pos.0 == self.row && pos.1 == self.column {
            //             return Ok(0);
            //         }
            //         return self.chording_click(pos.0, pos.1);
            //     }
            //     MouseState::ChordingNotFlag => {
            //         self.mouse_state = MouseState::DownUpAfterChording;
            //         self.right -= 1;
            //         if pos.0 == self.row && pos.1 == self.column {
            //             return Ok(0);
            //         }
            //         return self.chording_click(pos.0, pos.1);
            //     }
            //     _ => return Err(()),
            // },
            _ => return Err(()),
        }
        Ok(0)
    }
    /// 直接分析整局的操作流，中间也可以停顿
    /// 开始游戏前的任何操作也都记录次数
    pub fn step_flow(&mut self, operation: &Vec<(String, (usize, usize))>) -> Result<(), ()>
    where
        T: std::ops::Index<usize> + BoardSize + std::fmt::Debug + ReplaceBoard,
        T::Output: std::ops::Index<usize, Output = i32>,
    {
        for op in operation {
            self.step(&op.0, op.1)?;
        }
        Ok(())
    }
    /// 实施游戏状态事件
    pub fn step_game_state(&mut self, e: &str) -> Result<u8, ()>
    where
        T: std::ops::Index<usize> + BoardSize + std::fmt::Debug,
        T::Output: std::ops::Index<usize, Output = i32>,
    {
        match e {
            "replay" | "fail" => {
                self.game_board_state = GameBoardState::Loss;
                Ok(0)
            }
            _ => Err(()),
        }
    }
    fn is_win(&mut self) -> bool
    where
        T: std::ops::Index<usize> + BoardSize,
        T::Output: std::ops::Index<usize, Output = i32>,
    {
        for j in self.pointer_y..self.column {
            if self.game_board[self.pointer_x][j] < 10 {
                if self.game_board[self.pointer_x][j] != self.board[self.pointer_x][j] {
                    return false; // 安全性相关（发生作弊）
                }
            }
            if self.game_board[self.pointer_x][j] >= 10 && self.board[self.pointer_x][j] != -1 {
                self.pointer_y = j;
                return false;
            }
        }
        for i in self.pointer_x + 1..self.row {
            for j in 0..self.column {
                if self.game_board[i][j] < 10 {
                    if self.game_board[i][j] != self.board[i][j] {
                        return false; // 安全性相关（发生作弊）
                    }
                }
                if self.game_board[i][j] >= 10 && self.board[i][j] != -1 {
                    self.pointer_x = i;
                    self.pointer_y = j;
                    return false;
                }
            }
        }
        true
    }
    // 清空状态机里的点击次数
    fn clear_click_num(&mut self) {
        self.flag = 0;
        self.flaged_list.clear();
        self.double = 0;
        self.left = 0;
        self.right = 0;
        // 预标雷阶段的 ce 也要清空，否则会残留到正式游戏中
        self.lce = 0;
        self.rce = 0;
        self.dce = 0;
    }
}

/// 对比两次局面，返回新打开的格子
fn newly_opened(before: &Vec<Vec<i32>>, after: &Vec<Vec<i32>>) -> Vec<(usize, usize)> {
    let mut ans = vec![];
    for i in 0..before.len() {
        for j in 0..before[0].len() {
            if before[i][j] >= 10 && after[i][j] < 10 {
                ans.push((i, j));
            }
        }
    }
    ans
}

/// 鼠标状态
/// 做ffi传递时作如下约定：
/// UpUp => 1
/// UpDown => 2
/// UpDownNotFlag => 3
/// DownUp => 4
/// Chording => 5
/// ChordingNotFlag => 6
/// DownUpAfterChording => 7
/// Undefined => 8,
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MouseState {
    UpUp,
    UpDown,
    /// 右键按下，且既没有标雷，也没有取消标雷的状态
    UpDownNotFlag,
    DownUp,
    /// 双键都按下的其他状态
    Chording,
    /// 双键都按下，且是在不可以右击的格子上、先按下右键
    ChordingNotFlag,
    /// 双击后先弹起右键，左键还没弹起的状态
    DownUpAfterChording,
    /// 未初始化的状态
    Undefined,
}

/// 游戏局面状态
/// 做ffi传递时作如下约定：
/// Ready => 1
/// Playing => 2
/// Win => 3
/// Loss => 4
/// PreFlaging => 5
/// Display => 6,
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum GameBoardState {
    Ready,
    Playing,
    Win,
    Loss,
    /// 对界面有任何影响的操作（左键按下、标雷）发生后、第一次左键在未打开格子上弹起前的状态。此阶段将被记录到录像里
    PreFlaging,
    /// 局面作为录像在被播放
    Display,
}
