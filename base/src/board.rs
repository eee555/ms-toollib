use crate::utils::refresh_board;
use std::cmp::{max, min};

/// 局面类，分析操作与局面的交互
pub struct MinesweeperBoard {
    board: Vec<Vec<i32>>,
    gameBoard: Vec<Vec<i32>>,
    flagedList: Vec<(usize, usize)>, //记录哪些雷曾经被标过，则再标这些雷不记为ce
    left: usize,
    right: usize,
    chording: usize,
    ces: usize,
    flag: usize,
    solved3BV: usize,
    row: usize,
    column: usize,
    rightFlag: bool,    // 若rightFlag=True，则如果紧接着再chording就要把right减去1
    chordingFlag: bool, // chordingFlag=True，代表上一个时刻是双击弹起，此时再弹起左键或右键不做任何处理
}

impl MinesweeperBoard {
    pub fn new(board: Vec<Vec<i32>>) -> MinesweeperBoard {
        let row = board.len();
        let column = board[0].len();
        MinesweeperBoard {
            board: board,
            row: row,
            column: column,
            gameBoard: vec![vec![10; column]; row],
            left: 0,
            right: 0,
            chording: 0,
            ces: 0,
            flag: 0,
            solved3BV: 0,
            rightFlag: false,
            chordingFlag: false,
            flagedList: vec![],
        }
    }
    fn leftClick(&mut self, x: usize, y: usize) {
        self.left += 1;
        if self.gameBoard[x][y] != 10 {
            return;
        }
        match self.board[x][y] {
            0 => {
                self.solved3BV += 1;
                self.ces += 1;
                refresh_board(&self.board, &mut self.gameBoard, vec![(x, y)]);
                return;
            }
            -1 => {
                return;
            }
            _ => {
                refresh_board(&self.board, &mut self.gameBoard, vec![(x, y)]);
                if self.numIs3BV(x, y) {
                    self.solved3BV += 1;
                    self.ces += 1;
                    return;
                } else {
                    self.ces += 1;
                    return;
                }
            }
        }
    }
    fn rightClick(&mut self, x: usize, y: usize) {
        self.right += 1;
        if self.gameBoard[x][y] < 10 {
            return;
        } else {
            if self.board[x][y] != -1 {
                match self.gameBoard[x][y] {
                    10 => {
                        self.gameBoard[x][y] = 11;
                        self.flag += 1;
                    }
                    11 => {
                        self.gameBoard[x][y] = 10;
                        self.flag -= 1;
                    }
                    _ => return,
                }
                return;
            } else {
                match self.gameBoard[x][y] {
                    10 => {
                        self.gameBoard[x][y] = 11;
                        self.flag += 1;
                        self.flagedList.push((x, y));
                        let mut not_flag_flaged = true;
                        for flags in self.flagedList.clone() {
                            if x == flags.0 && y == flags.1 {
                                not_flag_flaged = false;
                                break;
                            }
                        }
                        if not_flag_flaged {
                            self.ces += 1;
                        }
                    }
                    11 => {
                        self.gameBoard[x][y] = 10;
                        self.flag -= 1;
                    }
                    _ => return,
                }
                return;
            }
        }
    }
    fn chordingClick(&mut self, x: usize, y: usize) {
        self.chording += 1;
        if self.gameBoard[x][y] == 0 || self.gameBoard[x][y] > 8 {
            return;
        }
        let mut flagChordingUseful = false; // 双击有效的基础上，周围是否有未打开的格子
        let mut chordingCells = vec![]; // 未打开的格子的集合
        let mut flagedNum = 0; // 双击点周围的标雷数
        let mut surround3BV = 0; // 周围的3BV
        let mut flag_ch_op = false; // 是否通过双击开空了：一次双击最多打开一个空
        for i in max(1, x) - 1..min(self.row, x + 2) {
            for j in max(1, y) - 1..min(self.column, y + 2) {
                if i != x || j != y {
                    if self.gameBoard[i][j] == 11 {
                        flagedNum += 1
                    }
                    if self.gameBoard[i][j] == 10 && self.board[i][j] != -1 {
                        if self.board[i][j] == 0 {
                            flag_ch_op = true;
                        }
                        flagChordingUseful = true;
                        chordingCells.push((i, j));
                        if self.numIs3BV(i, j) {
                            surround3BV += 1;
                        }
                    }
                }
            }
        }
        if flagedNum == self.gameBoard[x][y] && flagChordingUseful {
            self.ces += 1;
            self.solved3BV += surround3BV;
            if flag_ch_op {
                self.solved3BV += 1;
            }
            refresh_board(&self.board, &mut self.gameBoard, chordingCells);
        }
    }
    pub fn numIs3BV(&self, x: usize, y: usize) -> bool {
        // 判断该数字是不是3BV，0也可以
        if self.board[x][y] == -1 {
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
    pub fn step(&mut self, operation: Vec<(&str, (usize, usize))>) {
        for op in operation {
            match op.0 {
                "c1" => {
                    if self.rightFlag {
                        self.rightFlag = false;
                        self.right -= 1;
                    }
                }
                "l2" => {
                    if self.chordingFlag {
                        self.chordingFlag = false;
                    } else {
                        self.leftClick(op.1 .0, op.1 .1)
                    }
                }
                "r1" => self.rightClick(op.1 .0, op.1 .1),
                "c2" => {
                    self.chordingClick(op.1 .0, op.1 .1);
                    self.chordingFlag = true;
                }
                "r2" => {
                    if self.chordingFlag {
                        self.chordingFlag = false;
                    }
                    self.rightFlag = false; // 若rightFlag=True，则如果紧接着再chording就要把right减去1
                }
                _ => continue,
            }
        }
    }
    // pub fn reset(&self) {
    //     // 重载，暂时没用不写
    // }
}
