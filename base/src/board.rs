// 局面相关的类，录像在video

use crate::algorithms::{cal_possibility_onboard, solve_direct, solve_enumerate, solve_minus};
use crate::utils::{cal_bbbv_on_island, cal_cell_nums, cal_isl, cal_op, refresh_matrixs};

/// 静态游戏局面的包装类。  
/// 所有计算过的属性都会保存在这里。缓存计算结果的局面。  
#[derive(Clone, Debug)]
pub struct GameBoard {
    /// 游戏局面，来自玩家，上面标的雷可能是错的。
    pub game_board: Vec<Vec<i32>>,
    game_board_marked: Vec<Vec<i32>>,
    poss: Vec<Vec<f64>>,
    mine_num: usize,
    is_marked: bool, // game_board_marked是否被完全标记过
    has_poss: bool,  // 是否已经计算过概率
    basic_not_mine: Vec<(usize, usize)>,
    basic_is_mine: Vec<(usize, usize)>,
    enum_not_mine: Vec<(usize, usize)>,
    enum_is_mine: Vec<(usize, usize)>,
}

// impl Default for GameBoard {
//     fn default() -> Self {
//         GameBoard {
//             game_board: vec![],
//             game_board_marked: vec![],
//             poss: vec![],
//             mine_num: 0,
//             is_marked: false,
//             has_poss: false,
//             basic_not_mine: vec![],
//             basic_is_mine: vec![],
//             enum_not_mine: vec![],
//             enum_is_mine: vec![],
//         }
//     }
// }

impl GameBoard {
    pub fn new(mine_num: usize) -> GameBoard {
        GameBoard {
            game_board: vec![],
            game_board_marked: vec![],
            poss: vec![],
            mine_num: mine_num,
            is_marked: false,
            has_poss: false,
            basic_is_mine: vec![],
            basic_not_mine: vec![],
            enum_is_mine: vec![],
            enum_not_mine: vec![],
        }
    }
    pub fn set_game_board(&mut self, board: &Vec<Vec<i32>>) {
        let mut game_board_marked = board.clone();
        for i in 0..game_board_marked.len() {
            for j in 0..game_board_marked[0].len() {
                if game_board_marked[i][j] > 10 {
                    game_board_marked[i][j] = 10;
                }
            }
        }
        self.game_board = board.clone();
        self.game_board_marked = game_board_marked;
    }
    fn mark(&mut self) {
        // 一旦被标记，那么就会用3大判雷引擎都分析一遍
        // 相关参数都会计算并记录下来，is_marked也会改成true
        if self.is_marked {
            return;
        }
        let (mut a_s, mut x_s, mut b_s, _, _) = refresh_matrixs(&self.game_board_marked);
        let mut ans = solve_direct(&mut a_s, &mut x_s, &mut b_s, &mut self.game_board_marked)
            .unwrap()
            .0;
        self.basic_not_mine.append(&mut ans);

        let mut ans = solve_minus(&mut a_s, &mut x_s, &mut b_s, &mut self.game_board_marked)
            .unwrap()
            .0;
        self.basic_not_mine.append(&mut ans);
        for i in &self.basic_not_mine {
            self.game_board_marked[i.0][i.1] = 12;
        }
        for i in 0..self.game_board_marked.len() {
            for j in 0..self.game_board_marked[0].len() {
                if self.game_board_marked[i][j] == 11 {
                    self.basic_is_mine.push((i, j));
                }
            }
        }
        self.enum_not_mine = solve_enumerate(&a_s, &x_s, &b_s).0;
        // println!("yyyyyyyyyyyyyyyyy");
        for i in 0..self.game_board_marked.len() {
            for j in 0..self.game_board_marked[0].len() {
                if self.game_board_marked[i][j] == 11 && !self.basic_is_mine.contains(&(i, j)) {
                    self.enum_is_mine.push((i, j));
                }
            }
        }
        self.is_marked = true;
    }
    pub fn get_poss(&mut self) -> &Vec<Vec<f64>> {
        if !self.has_poss {
            self.mark();
            // println!("{:?}, {:?}", self.game_board_marked, self.mine_num);
            self.poss = cal_possibility_onboard(&self.game_board_marked, self.mine_num as f64)
                .unwrap()
                .0;
            self.has_poss = true;
        }
        &self.poss
    }
    pub fn get_basic_not_mine(&mut self) -> &Vec<(usize, usize)> {
        if !self.is_marked {
            self.mark();
            self.is_marked = true;
        }
        &self.basic_not_mine
    }
    pub fn get_basic_is_mine(&mut self) -> &Vec<(usize, usize)> {
        if !self.is_marked {
            self.mark();
            self.is_marked = true;
        }
        &self.basic_is_mine
    }
    pub fn get_enum_not_mine(&mut self) -> &Vec<(usize, usize)> {
        if !self.is_marked {
            self.mark();
            self.is_marked = true;
        }
        &self.enum_not_mine
    }
    pub fn get_enum_is_mine(&mut self) -> &Vec<(usize, usize)> {
        if !self.is_marked {
            self.mark();
            self.is_marked = true;
        }
        &self.enum_is_mine
    }
}

/// 静态局面的包装类。  
/// - 用途：筛选局面时，复杂的条件下，用于避免指标重复计算。  
/// - 局限：hizi缺少算法。  
/// 用Board类估算一亿局高级里有几个8的python代码如下：  
/// ``` python3
/// import ms_toollib as ms
/// cell8_num = 0
/// for i in range(100000000):
///     # 在第一行第一列起手，做标准埋雷
///     board = ms.laymine(row=16, column=30, mine_num=99, x0=0, y0=0)
///     # 包一下，准备计算属性
///     wrap_board = ms.Board(board)
///     cell8_num += wrap_board.cell8
/// print(f'数字8出现次数：{cell8_num}')
/// ```
#[derive(Clone)]
pub struct Board {
    pub board: Vec<Vec<i32>>,
    bbbv: usize,
    has_cal_bbbv: bool,
    openings: usize,
    has_cal_openings: bool,
    islands: usize,
    has_cal_islands: bool,
    cell0: usize,
    cell1: usize,
    cell2: usize,
    cell3: usize,
    cell4: usize,
    cell5: usize,
    cell6: usize,
    cell7: usize,
    cell8: usize,
    has_cal_cells: bool,
}

impl Board {
    pub fn new(board: Vec<Vec<i32>>) -> Board {
        Board {
            board: board,
            bbbv: 0,
            has_cal_bbbv: false,
            openings: 0,
            has_cal_openings: false,
            islands: 0,
            has_cal_islands: false,
            cell0: 0,
            cell1: 0,
            cell2: 0,
            cell3: 0,
            cell4: 0,
            cell5: 0,
            cell6: 0,
            cell7: 0,
            cell8: 0,
            has_cal_cells: false,
        }
    }
    pub fn get_bbbv(&mut self) -> usize {
        if self.has_cal_bbbv {
            return self.bbbv;
        }
        let a = cal_bbbv_on_island(&self.board);
        if !self.has_cal_openings {
            self.openings = cal_op(&self.board);
            self.has_cal_openings = true;
        }
        self.has_cal_bbbv = true;
        return a + self.openings;
    }
    pub fn get_op(&mut self) -> usize {
        if self.has_cal_openings {
            return self.openings;
        }
        self.has_cal_openings = true;
        return cal_op(&self.board);
    }
    pub fn get_isl(&mut self) -> usize {
        if self.has_cal_islands {
            return self.islands;
        }
        self.has_cal_islands = true;
        return cal_isl(&self.board);
    }
    fn cal_cell_nums(&mut self) {
        let ans = cal_cell_nums(&self.board);
        self.cell0 = ans[0];
        self.cell1 = ans[1];
        self.cell2 = ans[2];
        self.cell3 = ans[3];
        self.cell4 = ans[4];
        self.cell5 = ans[5];
        self.cell6 = ans[6];
        self.cell7 = ans[7];
        self.cell8 = ans[8];
        self.has_cal_cells = true;
    }
    pub fn get_cell0(&mut self) -> usize {
        if self.has_cal_cells {
            return self.cell0;
        }
        self.cal_cell_nums();
        return self.cell0;
    }
    pub fn get_cell1(&mut self) -> usize {
        if self.has_cal_cells {
            return self.cell1;
        }
        self.cal_cell_nums();
        return self.cell1;
    }
    pub fn get_cell2(&mut self) -> usize {
        if self.has_cal_cells {
            return self.cell2;
        }
        self.cal_cell_nums();
        return self.cell2;
    }
    pub fn get_cell3(&mut self) -> usize {
        if self.has_cal_cells {
            return self.cell3;
        }
        self.cal_cell_nums();
        return self.cell3;
    }
    pub fn get_cell4(&mut self) -> usize {
        if self.has_cal_cells {
            return self.cell4;
        }
        self.cal_cell_nums();
        return self.cell4;
    }
    pub fn get_cell5(&mut self) -> usize {
        if self.has_cal_cells {
            return self.cell5;
        }
        self.cal_cell_nums();
        return self.cell5;
    }
    pub fn get_cell6(&mut self) -> usize {
        if self.has_cal_cells {
            return self.cell6;
        }
        self.cal_cell_nums();
        return self.cell6;
    }
    pub fn get_cell7(&mut self) -> usize {
        if self.has_cal_cells {
            return self.cell7;
        }
        self.cal_cell_nums();
        return self.cell7;
    }
    pub fn get_cell8(&mut self) -> usize {
        if self.has_cal_cells {
            return self.cell8;
        }
        self.cal_cell_nums();
        return self.cell8;
    }
}
