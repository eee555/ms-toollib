use itertools::Itertools;
#[cfg(any(feature = "py", feature = "rs"))]
use rand::seq::SliceRandom;
#[cfg(any(feature = "py", feature = "rs"))]
use rand::thread_rng;
use std::cmp::{max, min};
use std::vec;
// use std::convert::TryInto;
#[cfg(feature = "js")]
use getrandom::getrandom;

use crate::safe_board;
use crate::safe_board::BoardSize;
use crate::ENUM_LIMIT;

// 整个模块是最底层的一些小工具，如埋雷、局面分块、计算3BV等

/// 输入局面，计算空，即0的8连通域数
pub fn cal_op<T>(board_raw: &T) -> usize
where
    T: std::ops::Index<usize> + safe_board::BoardSize,
    T::Output: std::ops::Index<usize, Output = i32>,
{
    let row = board_raw.get_row();
    let column = board_raw.get_column();
    let mut board = vec![vec![0; column]; row];
    for i in 0..row {
        for j in 0..column {
            board[i][j] = board_raw[i][j];
        }
    }
    let mut op = 0;
    for i in 0..row {
        for j in 0..column {
            if board[i][j] == 0 {
                infect_board(&mut board, i, j);
                op += 1;
            }
        }
    }
    op
}

// Board(x, y)位置的整个空都用数字1填满，仅计算Op用
fn infect_board(board: &mut Vec<Vec<i32>>, x: usize, y: usize) {
    let row = board.len();
    let column = board[0].len();
    board[x][y] = 1;
    for i in max(1, x) - 1..min(row, x + 2) {
        for j in max(1, y) - 1..min(column, y + 2) {
            if board[i][j] == 0 {
                infect_board(board, i, j);
            }
        }
    }
}

/// 输入局面，计算岛  
pub fn cal_isl<T>(raw_board: &T) -> usize
where
    T: std::ops::Index<usize> + safe_board::BoardSize,
    T::Output: std::ops::Index<usize, Output = i32>,
{
    let row = raw_board.get_row();
    let column = raw_board.get_column();
    let mut board = vec![vec![1; column]; row];
    for i in 0..row {
        for j in 0..column {
            if raw_board[i][j] <= 0 {
                continue;
            }
            let mut flag = true;
            'outer: for m in max(1, i) - 1..min(row, i + 2) {
                for n in max(1, j) - 1..min(column, j + 2) {
                    if raw_board[m][n] == 0 {
                        flag = false;
                        break 'outer;
                    }
                }
            }
            if flag {
                board[i][j] = 0;
            }
        }
    }
    cal_op(&board)
}

/// 计算每个数字出现的次数  
pub fn cal_cell_nums<T>(raw_board: &T) -> [usize; 9]
where
    T: std::ops::Index<usize> + BoardSize,
    T::Output: std::ops::Index<usize, Output = i32>,
{
    let row = raw_board.get_row();
    let column = raw_board.get_column();
    let mut ans = [0; 9];
    for i in 0..row {
        for j in 0..column {
            if raw_board[i][j] < 0 {
                continue;
            }
            ans[raw_board[i][j] as usize] += 1;
        }
    }
    ans
}

/// 根据游戏局面生成矩阵，不分块。输入必须保证是合法的游戏局面。  
/// - 注意：优点是含义明确，便于理解。但不分块
pub fn refresh_matrix(
    game_board: &Vec<Vec<i32>>,
) -> (Vec<Vec<i32>>, Vec<(usize, usize)>, Vec<i32>) {
    let row = game_board.len();
    let column = game_board[0].len();
    let mut MatrixA: Vec<Vec<i32>> = Vec::new();
    let mut Matrixx: Vec<(usize, usize)> = Vec::new();
    let mut Matrixb: Vec<i32> = Vec::new();
    let mut MatrixARowNum = 0;
    let mut MatrixAColumnNum = 0;

    for i in 0..row {
        for j in 0..column {
            if game_board[i][j] > 0 && game_board[i][j] < 10 {
                let mut flag: bool = false;
                for m in max(1, i) - 1..min(row, i + 2) {
                    for n in max(1, j) - 1..min(column, j + 2) {
                        if game_board[m][n] == 10 {
                            flag = true;
                        }
                    }
                }
                if flag {
                    MatrixA.push(vec![0; MatrixAColumnNum]);
                    Matrixb.push(game_board[i][j]);
                    MatrixARowNum += 1;
                    for m in max(1, i) - 1..min(row, i + 2) {
                        for n in max(1, j) - 1..min(column, j + 2) {
                            if game_board[m][n] == 11 {
                                Matrixb[MatrixARowNum - 1] -= 1
                            } else if game_board[m][n] == 10 {
                                let mut flag_exit: bool = false;
                                for idMatrixx in 0..MatrixAColumnNum {
                                    if Matrixx[idMatrixx].0 == m && Matrixx[idMatrixx].1 == n {
                                        flag_exit = true;
                                        MatrixA[MatrixARowNum - 1][idMatrixx] = 1;
                                    }
                                }
                                if !flag_exit {
                                    for ii in 0..MatrixARowNum {
                                        MatrixA[ii].push(0)
                                    }
                                    Matrixx.push((m, n));
                                    MatrixAColumnNum += 1;
                                    MatrixA[MatrixARowNum - 1][MatrixAColumnNum - 1] = 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    (MatrixA, Matrixx, Matrixb)
}

/// 根据游戏局面生成矩阵，分段。输入的必须保证是合法的游戏局面。  
/// - *基于数字生成，矩阵的行可能有重复。  
pub fn refresh_matrixs(
    board_of_game: &Vec<Vec<i32>>,
) -> (
    Vec<Vec<Vec<i32>>>,
    Vec<Vec<(usize, usize)>>,
    Vec<Vec<i32>>,
    usize,
    usize,
) {
    // 根据游戏局面分块生成矩阵。分段的数据结构是最外面再套一层Vec
    // board_of_game必须且肯定是正确标雷的游戏局面，但不需要标全，不能标非雷
    // 矩阵的行和列都可能有重复
    // unknow_block是未知格子数量, is_minenum是标出的是雷的数量
    let row = board_of_game.len();
    let column = board_of_game[0].len();
    let mut unknow_block = 0;
    let mut is_minenum = 0;
    let mut matrix_as = vec![];
    let mut matrix_xs = vec![];
    let mut matrix_bs = vec![];
    let mut all_cell: Vec<(usize, usize)> = vec![]; // 记录所有周围有未打开格子的数字的位置
    for i in 0..row {
        for j in 0..column {
            if board_of_game[i][j] >= 0 && board_of_game[i][j] < 10 {
                'outer: for m in max(1, i) - 1..min(row, i + 2) {
                    for n in max(1, j) - 1..min(column, j + 2) {
                        if board_of_game[m][n] == 10 {
                            all_cell.push((i, j));
                            break 'outer;
                        }
                    }
                }
            } else if board_of_game[i][j] == 10 {
                // 数内部有几个格子
                let mut flag = true;
                for m in max(1, i) - 1..min(row, i + 2) {
                    for n in max(1, j) - 1..min(column, j + 2) {
                        if board_of_game[m][n] < 10 {
                            flag = false;
                        }
                    }
                }
                if flag {
                    unknow_block += 1;
                }
            } else if board_of_game[i][j] == 11 {
                is_minenum += 1;
            }
        }
    }
    let mut p = 0; //指针，代表第几块
                   // println!("{:?}", all_cell);
    while !all_cell.is_empty() {
        matrix_xs.push(vec![]);
        matrix_bs.push(vec![]);
        let x_0 = all_cell[0].0;
        let y_0 = all_cell[0].1;
        let mut num_cells = vec![]; // 记录了当前段的数字格的坐标
        let mut temp_cells = vec![]; // 记录了待查找的数字格的坐标
        let mut flag_num = 0;
        for m in max(1, x_0) - 1..min(row, x_0 + 2) {
            for n in max(1, y_0) - 1..min(column, y_0 + 2) {
                if board_of_game[m][n] == 10 {
                    matrix_xs[p].push((m, n));
                }
                if board_of_game[m][n] == 11 {
                    flag_num += 1;
                }
            }
        }
        matrix_bs[p].push(board_of_game[x_0][y_0] - flag_num);
        num_cells.push((x_0, y_0));
        temp_cells.push((x_0, y_0));
        while !temp_cells.is_empty() {
            let x_e = temp_cells[0].0;
            let y_e = temp_cells[0].1;
            temp_cells.remove(0);
            for t in (1..all_cell.len()).rev() {
                // 从temp_cells中拿出一个格子，找出与其相邻的所有格子，加入temp_cells和matrix_bs、matrix_xs
                let x_t = all_cell[t].0;
                let y_t = all_cell[t].1;
                if x_t >= x_e + 3 || x_e >= x_t + 3 || y_t >= y_e + 3 || y_e >= y_t + 3 {
                    continue;
                }
                let mut flag_be_neighbor = false;
                for m in max(1, max(x_t, x_e)) - 1..min(row, min(x_t + 2, x_e + 2)) {
                    for n in max(1, max(y_t, y_e)) - 1..min(column, min(y_t + 2, y_e + 2)) {
                        if board_of_game[m][n] == 10 {
                            flag_be_neighbor = true;
                            break;
                        }
                    }
                } // 从局面上看，如果两个数字有相同的未知格子，那么就会分到同一块
                if flag_be_neighbor {
                    let mut flag_num = 0;
                    for m in max(1, x_t) - 1..min(row, x_t + 2) {
                        for n in max(1, y_t) - 1..min(column, y_t + 2) {
                            if board_of_game[m][n] == 10 {
                                if !matrix_xs[p].contains(&(m, n)) {
                                    matrix_xs[p].push((m, n));
                                }
                            }
                            if board_of_game[m][n] == 11 {
                                flag_num += 1;
                            }
                        }
                    }
                    matrix_bs[p].push(board_of_game[x_t][y_t] - flag_num);
                    num_cells.push((x_t, y_t));
                    temp_cells.push(all_cell[t]);
                    all_cell.remove(t);
                }
            }
        }
        matrix_as.push(vec![vec![0; matrix_xs[p].len()]; matrix_bs[p].len()]);
        for i in 0..num_cells.len() {
            for j in 0..matrix_xs[p].len() {
                if num_cells[i].0 <= matrix_xs[p][j].0 + 1
                    && matrix_xs[p][j].0 <= num_cells[i].0 + 1
                    && num_cells[i].1 <= matrix_xs[p][j].1 + 1
                    && matrix_xs[p][j].1 <= num_cells[i].1 + 1
                {
                    matrix_as[p][i][j] = 1;
                }
            }
        }
        all_cell.remove(0);
        p += 1;
    }
    (matrix_as, matrix_xs, matrix_bs, unknow_block, is_minenum)
}

/// 根据游戏局面生成矩阵，分段、且分块。输入的必须保证是合法的游戏局面。  
pub fn refresh_matrixses(
    board_of_game: &Vec<Vec<i32>>,
) -> (
    Vec<Vec<Vec<Vec<i32>>>>,
    Vec<Vec<Vec<(usize, usize)>>>,
    Vec<Vec<Vec<i32>>>,
) {
    let row = board_of_game.len();
    let column = board_of_game[0].len();
    let mut Ases = vec![];
    let mut xses = vec![];
    let mut bses = vec![];
    let (mut As, mut xs, mut bs, _, _) = refresh_matrixs(board_of_game);
    if As.len() == 1 {
        // 不可能为0，至少为1
        return (vec![As], vec![xs], vec![bs]);
    }
    // 邻接矩阵
    let mut adjacency_matrix = vec![vec![false; As.len()]; As.len()];
    // 局面的复刻，用于标记遍历过的格子
    let mut board_mark = board_of_game.clone();
    let mut cell_10 = vec![];
    for i in 0..row {
        for j in 0..column {
            if board_mark[i][j] == 10 {
                board_mark[i][j] = 21;
                let mut flag_c = false;
                let mut buffer = vec![];
                buffer.push((i, j));
                // 标志是否搜索完的缓冲区
                while !buffer.is_empty() {
                    let (t_i, t_j) = buffer.pop().unwrap();
                    let mut flag_is_side = false;
                    for m in max(1, t_i) - 1..min(row, t_i + 2) {
                        for n in max(1, t_j) - 1..min(column, t_j + 2) {
                            if board_mark[m][n] == 10 {
                                board_mark[m][n] = 21;
                                buffer.push((m, n));
                            } else if board_mark[m][n] < 10 {
                                flag_is_side = true;
                            }
                        }
                    }
                    if flag_is_side {
                        if !flag_c {
                            cell_10.push(vec![]);
                            flag_c = true;
                        }
                        cell_10.last_mut().unwrap().push((t_i, t_j));
                    }
                }
            } else {
                continue;
            }
        }
    }
    // println!("{:?}", cell_10);
    if cell_10.len() == 1 {
        // 不可能为0，至少为1
        return (vec![As], vec![xs], vec![bs]);
    }
    for mut block in cell_10 {
        let mut seed_id = -1;
        while !block.is_empty() {
            let seed = block.pop().unwrap();
            let t = xs.iter().position(|r| r.contains(&seed)).unwrap();
            if seed_id >= 0 {
                adjacency_matrix[seed_id as usize][t] = true;
                adjacency_matrix[t][seed_id as usize] = true;
            }
            seed_id = t as i32;
            block.retain(|x| !xs[seed_id as usize].contains(x))
        }
    } // 整理完邻接矩阵。无向图。
    for i in 0..As.len() {
        if As[i].is_empty() {
            continue;
        }
        Ases.push(vec![]);
        xses.push(vec![]);
        bses.push(vec![]);
        let mut buffer = vec![i];

        while !buffer.is_empty() {
            let t = buffer.pop().unwrap();
            Ases.last_mut().unwrap().push(vec![]);
            Ases.last_mut()
                .unwrap()
                .last_mut()
                .unwrap()
                .append(&mut As[t]);
            xses.last_mut().unwrap().push(vec![]);
            xses.last_mut()
                .unwrap()
                .last_mut()
                .unwrap()
                .append(&mut xs[t]);
            bses.last_mut().unwrap().push(vec![]);
            bses.last_mut()
                .unwrap()
                .last_mut()
                .unwrap()
                .append(&mut bs[t]);
            for idj in t..As.len() {
                if adjacency_matrix[t][idj] {
                    buffer.push(idj);
                }
            }
        }
    }
    (Ases, xses, bses)
}

// 获取0~limit-1范围内的随机整数
// 用于js平台
#[cfg(feature = "js")]
pub fn get_random_int(limit: usize) -> usize {
    if limit == 0 {
        return 0;
    }
    let mut a = [0, 0, 0, 0];
    let mut t;
    loop {
        getrandom(&mut a).unwrap();
        // println!("{:?}", a);
        t = (a[0] as usize) << 24 ^ (a[1] as usize) << 16 ^ (a[2] as usize) << 8 ^ (a[3] as usize);
        if t < (0b11111111_11111111_11111111_11111111 / limit * limit) {
            break;
        }
    }
    t % limit
}

#[cfg(feature = "js")]
pub trait js_shuffle {
    fn shuffle_(&mut self);
}

#[cfg(feature = "js")]
impl js_shuffle for Vec<i32> {
    fn shuffle_(&mut self) {
        // 存疑！！！！！
        let l = self.len();
        for i in 1..l {
            let id = get_random_int(i + 1);
            let t = self[i];
            self[i] = self[id];
            self[id] = t;
        }
    }
}

/// 一维埋雷，给局部埋雷，完全随机。
/// - 需要埋雷的区域的面积，雷数。
/// - 例如，高级标准埋雷时，area = 16*30-1
pub fn get_board_1d(area: usize, minenum: usize) -> Vec<i32> {
    let mut board_1d: Vec<i32> = vec![];
    board_1d.reserve(area);
    board_1d = vec![0; area - minenum];
    board_1d.append(&mut vec![-1; minenum]);
    #[cfg(any(feature = "py", feature = "rs"))]
    let mut rng = thread_rng();

    #[cfg(any(feature = "py", feature = "rs"))]
    board_1d.shuffle(&mut rng);

    #[cfg(feature = "js")]
    board_1d.shuffle_();
    board_1d
}

/// 根据起手不开空的规则，把一维的局面转换成二维的。
pub fn trans_board_1d_2d_op(
    board_1d: &Vec<i32>,
    row: usize,
    column: usize,
    x0: usize,
    y0: usize,
) -> Vec<Vec<i32>> {
    let mut board = vec![vec![0; column]; row];
    let mut i = 0;
    for x in 0..row {
        for y in 0..column {
            if x <= x0 + 1 && x0 <= x + 1 && y <= y0 + 1 && y0 <= y + 1 {
                continue;
            }
            if board_1d[i] < 0 {
                board[x][y] = -1;
                for j in max(1, x) - 1..min(row, x + 2) {
                    for k in max(1, y) - 1..min(column, y + 2) {
                        if board[j][k] >= 0 {
                            board[j][k] += 1
                        }
                    }
                }
            }
            i += 1;
        }
    }
    board
}

/// 通用标准埋雷引擎。
/// - 标准埋雷规则：起手位置非雷，其余位置的雷服从均匀分布。
/// - 输出：二维的局面，其中0代表空，1~8代表1~8，-1代表雷。
pub fn laymine(row: usize, column: usize, minenum: usize, x0: usize, y0: usize) -> Vec<Vec<i32>> {
    let board1_dim = get_board_1d(row * column - 1, minenum);

    let mut board1_dim_2: Vec<i32> = vec![];
    board1_dim_2.reserve(row * column);
    let pointer = x0 + y0 * row;
    for i in 0..pointer {
        board1_dim_2.push(board1_dim[i]);
    }
    board1_dim_2.push(0);
    for i in pointer..(row * column - 1) {
        board1_dim_2.push(board1_dim[i]);
    }
    let mut board: Vec<Vec<i32>> = vec![vec![0; column]; row];
    for i in 0..(row * column) {
        if board1_dim_2[i] < 0 {
            let x = i % row;
            let y = i / row;
            board[x][y] = -1;
            for j in max(1, x) - 1..min(row, x + 2) {
                for k in max(1, y) - 1..min(column, y + 2) {
                    if board[j][k] >= 0 {
                        board[j][k] += 1;
                    }
                }
            }
        }
    }
    board
}

/// 通用win7规则埋雷引擎。
/// - win7规则：起手位置开空，其余位置的雷服从均匀分布。
/// - 输出：二维的局面，其中0代表空，1~8代表1~8，-1代表雷。
pub fn laymine_op(
    row: usize,
    column: usize,
    minenum: usize,
    x0: usize,
    y0: usize,
) -> Vec<Vec<i32>> {
    let mut area_op = 9;
    if x0 == 0 || y0 == 0 || x0 == row - 1 || y0 == column - 1 {
        if x0 == 0 && y0 == 0
            || x0 == 0 && y0 == column - 1
            || x0 == row - 1 && y0 == 0
            || x0 == row - 1 && y0 == column - 1
        {
            area_op = 4;
        } else {
            area_op = 6;
        }
    }
    let area = row * column - area_op;
    let board_1d = get_board_1d(area, minenum);
    trans_board_1d_2d_op(&board_1d, row, column, x0, y0)
}

pub fn cal_bbbv_on_island<T>(board: &T) -> usize
where
    T: std::ops::Index<usize> + safe_board::BoardSize,
    T::Output: std::ops::Index<usize, Output = i32>,
{
    // 计算除空以外的3BV
    let row = board.get_row();
    let column = board.get_column();
    let mut Num3BVonIsland = 0;
    for i in 0..row {
        for j in 0..column {
            if board[i][j] > 0 {
                let mut flag: bool = true;
                for x in max(1, i) - 1..min(row, i + 2) {
                    for y in max(1, j) - 1..min(column, j + 2) {
                        if board[x][y] == 0 {
                            flag = false;
                        }
                    }
                }
                if flag {
                    Num3BVonIsland += 1;
                }
            }
        }
    }
    Num3BVonIsland
}

/// 计算局面的3BV
pub fn cal_bbbv<T>(board: &T) -> usize
where
    T: std::ops::Index<usize> + safe_board::BoardSize,
    T::Output: std::ops::Index<usize, Output = i32>,
{
    cal_bbbv_on_island(board) + cal_op(board)
}

/// 依据左击位置刷新局面。如踩雷，标上或14、15标记
/// - 注意：兼容12标记符
pub fn refresh_board<T>(
    board: &T,
    boardofGame: &mut Vec<Vec<i32>>,
    mut clicked_poses: Vec<(usize, usize)>,
) where
    T: std::ops::Index<usize> + safe_board::BoardSize,
    T::Output: std::ops::Index<usize, Output = i32>,
{
    let row = board.get_row();
    let column = board.get_column();
    // 是否踩雷
    let mut loss_flag = false;
    while let Some(top) = clicked_poses.pop() {
        let (i, j) = top;
        if board[i][j] > 0 {
            boardofGame[i][j] = board[i][j];
        } else if board[i][j] == 0 {
            boardofGame[i][j] = 0;
            for m in max(1, i) - 1..min(row, i + 2) {
                for n in max(1, j) - 1..min(column, j + 2) {
                    if (i != m || j != n) && (boardofGame[m][n] == 10 || boardofGame[m][n] == 12) {
                        clicked_poses.push((m, n));
                    }
                }
            }
        } else {
            boardofGame[i][j] = 15; // 标红雷，此处是雷，且踩到了
            loss_flag = true;
        }
    }
    // 标叉雷
    if loss_flag {
        for i in 0..row {
            for j in 0..column {
                if boardofGame[i][j] == 11 && board[i][j] != -1 {
                    boardofGame[i][j] = 14; // 叉雷，即标错的雷
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct BigNumber {
    // 科学计数法表示的大数字
    // 必定大于等于1，a必定满足小于10大于等于1
    pub a: f64,
    pub b: i32,
}

impl BigNumber {
    fn a_become_smaller_than(&mut self, thrd: f64) {
        // 如果big_number大于thrd
        // 把位数都放到指数上，使其满足a小于10大于等于1
        if self.a < thrd {
            return;
        }
        while self.a >= 10.0 {
            self.a /= 10.0;
            self.b += 1;
        }
    }
    pub fn mul_usize(&mut self, k: usize) {
        // 与usize相乘
        if k == 0 {
            self.a = 0.0;
            self.b = 1;
        } else {
            self.a *= k as f64;
            self.a_become_smaller_than(10.0);
        }
    }
    pub fn mul_big_number(&mut self, k: &BigNumber) {
        // 与big_number相乘, big_number必须至少为1
        self.a *= k.a;
        self.b += k.b;
        self.a_become_smaller_than(10.0);
    }
    pub fn add_big_number(&mut self, k: &BigNumber) {
        let mut ka = k.a;
        let mut kb = k.b;
        while self.b > kb {
            ka /= 10.0;
            kb += 1;
        }
        while self.b < kb {
            self.a /= 10.0;
            self.b += 1;
        }
        self.a += ka;
        self.a_become_smaller_than(10.0);
    }
    pub fn div_big_num(&mut self, k: &BigNumber) -> f64 {
        // 计算大数相除
        let mut ans = self.a / k.a;
        while self.b < k.b {
            ans /= 10.0;
            self.b += 1;
        }
        while self.b > k.b {
            ans *= 10.0;
            self.b -= 1;
        }
        ans
    }
    pub fn div_usize(&mut self, k: usize) {
        // 计算大数除以正整数。这里被除数大于等于0；除数大于等于1
        if self.a < 1e-8 && self.b == 1 {
            return;
        } else {
            self.a /= k as f64;
            while self.a < 1.0 {
                self.a *= 10.0;
                self.b -= 1;
            }
        }
    }
}

pub fn C(n: usize, k: usize) -> BigNumber {
    // n不超过1e10
    if n < k + k {
        return C(n, n - k);
    };
    let maximum_limit: f64 = 1e208;
    let mut c = BigNumber { a: 1.0, b: 0 };
    for i in 0..k {
        c.a *= (n - i) as f64;
        c.a /= (i + 1) as f64;
        c.a_become_smaller_than(maximum_limit);
    }
    c.a_become_smaller_than(10.0);
    c
}

pub fn C_query<T, U>(n: T, k: U) -> usize
where
    T: Into<usize>,
    U: Into<usize>,
{
    // 查表计算8以内小数字的组合数
    let a = [
        [1, 0, 0, 0, 0, 0, 0, 0, 0],
        [1, 1, 0, 0, 0, 0, 0, 0, 0],
        [1, 2, 1, 0, 0, 0, 0, 0, 0],
        [1, 3, 3, 1, 0, 0, 0, 0, 0],
        [1, 4, 6, 4, 1, 0, 0, 0, 0],
        [1, 5, 10, 10, 5, 1, 0, 0, 0],
        [1, 6, 15, 20, 15, 6, 1, 0, 0],
        [1, 7, 21, 35, 35, 21, 7, 1, 0],
        [1, 8, 28, 56, 70, 56, 28, 8, 1],
    ];
    a[n.into()][k.into()]
}

pub fn combine(
    MatrixA: &Vec<Vec<i32>>,
    Matrixx: &Vec<(usize, usize)>,
) -> (Vec<Vec<i32>>, Vec<(usize, usize)>, Vec<Vec<usize>>) {
    // 检查地位完全相同的格子，全部返回。例如[[3,1,2],[0,5],[4],[6]]
    // MatrixA不能为空
    // 并在内部更改矩阵，合并重复的列
    let mut matrixA_squeeze = MatrixA.clone();
    let mut matrixx_squeeze = Matrixx.clone();
    let cells_num = matrixx_squeeze.len();
    let mut pair_cells = vec![];
    let mut del_cells = vec![]; // 由于重复需要最后被删除的列
    for i in 0..cells_num {
        pair_cells.push(vec![i]);
        for j in i + 1..cells_num {
            if !matrixA_squeeze.iter().any(|x| x[i] != x[j]) {
                pair_cells[i].push(j);
                del_cells.push(j);
            }
        }
    }
    del_cells.sort_by(|a, b| b.cmp(&a));
    del_cells.dedup();
    for i in del_cells {
        for r in 0..matrixA_squeeze.len() {
            matrixA_squeeze[r].remove(i);
        }
        matrixx_squeeze.remove(i);
        pair_cells.remove(i);
    }
    let cell_squeeze_num = pair_cells.len();
    for i in 0..cell_squeeze_num {
        let k = pair_cells[i].len() as i32;
        for r in 0..matrixA_squeeze.len() {
            matrixA_squeeze[r][i] *= k;
        }
    }
    (matrixA_squeeze, matrixx_squeeze, pair_cells)
}


/// 枚举法求解矩阵，返回所有的解
pub fn cal_all_solution(
    matrixA: &Vec<Vec<i32>>,
    Matrixb: &Vec<i32>,
) -> Vec<Vec<u8>> {
    let column = matrixA[0].len();
    let row = matrixA.len();
    let mut enum_comb_table: Vec<Vec<u8>> = vec![vec![0; column]];
    let mut not_enum_cell: Vec<bool> = vec![true; column]; // 记录每个位置是否被枚举过，true是没有被枚举过
    let mut enum_cell_table: Vec<Vec<usize>> = vec![];
    for row in 0..row {
        let mut new_enum_cell = vec![]; // 当前条件涉及的新格子
        let mut enum_cell = vec![]; // 当前条件涉及的所有格子
        let mut new_enum_max = vec![];
        for j in 0..column {
            if matrixA[row][j] > 0 {
                enum_cell.push(j);
                if not_enum_cell[j] {
                    not_enum_cell[j] = false;
                    new_enum_cell.push(j);
                    new_enum_max.push(matrixA[row][j]);
                }
            }
        }
        // 第一步，整理出当前条件涉及的所有格子，以及其中哪些是新格子
        let mut new_enum_table = (0..new_enum_cell.len())
            .map(|i| 0..new_enum_max[i] + 1)
            .multi_cartesian_product()
            .collect::<Vec<_>>();
        new_enum_table.retain(|x| x.iter().sum::<i32>() <= Matrixb[row]);
        // 第二步，获取这些新枚举到的格子的所有满足周围雷数约束的情况，即子枚举表
        if new_enum_table.is_empty() {
            enum_comb_table.retain(|item| {
                enum_cell
                    .iter()
                    .fold(0, |sum: u8, i: &usize| sum + item[*i])
                    == Matrixb[row] as u8
            });
        // 第三步，若子枚举表为空，不用将子枚举表与主枚举表合并；且只检查主枚举表是否满足当前这条规则，删除一些不满足的
        } else {
            let mut flag_1 = true; // 代表新枚举的格子是否需要新增情况
            let enum_comb_table_len = enum_comb_table.len();
            for item in new_enum_table {
                if flag_1 {
                    for m in 0..new_enum_cell.len() {
                        for n in 0..enum_comb_table_len {
                            enum_comb_table[n][new_enum_cell[m]] = item[m] as u8;
                        }
                    }
                    flag_1 = false;
                } else {
                    for n in 0..enum_comb_table_len {
                        let mut one_row_in_new_table = enum_comb_table[n].clone();
                        for m in 0..new_enum_cell.len() {
                            one_row_in_new_table[new_enum_cell[m]] = item[m] as u8;
                        }
                        enum_comb_table.push(one_row_in_new_table);
                    }
                }
            } // 第四步，若子枚举表非空，先将子枚举表与主枚举表合并
            let mut equations = vec![];
            for kk in &enum_cell {
                for rr in 0..row {
                    if matrixA[rr][*kk] > 0 {
                        equations.push(rr);
                    }
                }
            }
            equations.dedup();
            // 第五步，再找出本条规则涉及的之前所有的规则的id
            for equ in equations {
                enum_comb_table.retain(|item| {
                    enum_cell_table[equ]
                        .iter()
                        .fold(0, |sum: u8, i: &usize| sum + item[*i])
                        == Matrixb[equ] as u8
                });
            }
            enum_comb_table.retain(|item| {
                enum_cell
                    .iter()
                    .fold(0, |sum: u8, i: &usize| sum + item[*i])
                    == Matrixb[row] as u8
            }); // 这段重复了，不过不影响性能，之后优化
                // 第六步，用本条规则、以及涉及的之前所有规则过滤所有情况
        }
        enum_cell_table.push(enum_cell);
    }
    enum_comb_table
}

// pub fn enum_comb(
//     matrixA_squeeze: &Vec<Vec<i32>>,
//     matrixx_squeeze: &Vec<(usize, usize)>,
//     Matrixb: &Vec<i32>,
// ) -> Vec<Vec<u8>> {
//     // 拟弃用
//     // 枚举法求解矩阵，返回所有的解
//     let column = matrixx_squeeze.len();
//     let row = matrixA_squeeze.len();
//     let mut enum_comb_table: Vec<Vec<u8>> = vec![vec![0; column]];
//     let mut not_enum_cell: Vec<bool> = vec![true; column]; // 记录每个位置是否被枚举过，true是没有被枚举过
//     let mut enum_cell_table: Vec<Vec<usize>> = vec![];
//     for row in 0..row {
//         let mut new_enum_cell = vec![]; // 当前条件涉及的新格子
//         let mut enum_cell = vec![]; // 当前条件涉及的所有格子
//         let mut new_enum_max = vec![];
//         for j in 0..column {
//             if matrixA_squeeze[row][j] > 0 {
//                 enum_cell.push(j);
//                 if not_enum_cell[j] {
//                     not_enum_cell[j] = false;
//                     new_enum_cell.push(j);
//                     new_enum_max.push(matrixA_squeeze[row][j]);
//                 }
//             }
//         }
//         // 第一步，整理出当前条件涉及的所有格子，以及其中哪些是新格子
//         let mut new_enum_table = (0..new_enum_cell.len())
//             .map(|i| 0..new_enum_max[i] + 1)
//             .multi_cartesian_product()
//             .collect::<Vec<_>>();
//         new_enum_table.retain(|x| x.iter().sum::<i32>() <= Matrixb[row]);
//         // 第二步，获取这些新枚举到的格子的所有满足周围雷数约束的情况，即子枚举表
//         if new_enum_table.is_empty() {
//             enum_comb_table.retain(|item| {
//                 enum_cell
//                     .iter()
//                     .fold(0, |sum: u8, i: &usize| sum + item[*i])
//                     == Matrixb[row] as u8
//             });
//         // 第三步，若子枚举表为空，不用将子枚举表与主枚举表合并；且只检查主枚举表是否满足当前这条规则，删除一些不满足的
//         } else {
//             let mut flag_1 = true; // 代表新枚举的格子是否需要新增情况
//             let enum_comb_table_len = enum_comb_table.len();
//             for item in new_enum_table {
//                 if flag_1 {
//                     for m in 0..new_enum_cell.len() {
//                         for n in 0..enum_comb_table_len {
//                             enum_comb_table[n][new_enum_cell[m]] = item[m] as u8;
//                         }
//                     }
//                     flag_1 = false;
//                 } else {
//                     for n in 0..enum_comb_table_len {
//                         let mut one_row_in_new_table = enum_comb_table[n].clone();
//                         for m in 0..new_enum_cell.len() {
//                             one_row_in_new_table[new_enum_cell[m]] = item[m] as u8;
//                         }
//                         enum_comb_table.push(one_row_in_new_table);
//                     }
//                 }
//             } // 第四步，若子枚举表非空，先将子枚举表与主枚举表合并
//             let mut equations = vec![];
//             for kk in &enum_cell {
//                 for rr in 0..row {
//                     if matrixA_squeeze[rr][*kk] > 0 {
//                         equations.push(rr);
//                     }
//                 }
//             }
//             equations.dedup();
//             // 第五步，再找出本条规则涉及的之前所有的规则的id
//             for equ in equations {
//                 enum_comb_table.retain(|item| {
//                     enum_cell_table[equ]
//                         .iter()
//                         .fold(0, |sum: u8, i: &usize| sum + item[*i])
//                         == Matrixb[equ] as u8
//                 });
//             }
//             enum_comb_table.retain(|item| {
//                 enum_cell
//                     .iter()
//                     .fold(0, |sum: u8, i: &usize| sum + item[*i])
//                     == Matrixb[row] as u8
//             }); // 这段重复了，不过不影响性能，之后优化
//                 // 第六步，用本条规则、以及涉及的之前所有规则过滤所有情况
//         }
//         enum_cell_table.push(enum_cell);
//     }
//     enum_comb_table
// }

// fn enumerateSub(Col: usize, minenum: usize) -> Vec<Vec<usize>> {
//     let mut Out: Vec<Vec<usize>> = vec![];
//     for i in (0..Col).combinations(minenum) {
//         Out.push(vec![0; Col]);
//         let len = Out.len() - 1;
//         for j in 0..minenum {
//             Out[len][i[j]] = 1;
//         }
//     }
//     Out
// }

/// 忘了干嘛用的，有待重构。和弱可猜有关。
// pub fn enuOneStep(mut AllTable: Vec<Vec<usize>>, TableId: Vec<usize>, b: i32) -> Vec<Vec<usize>> {
//     // AllTable不能为空
//     let mut NewId: Vec<usize> = vec![];
//     for i in &TableId {
//         if AllTable[0][*i] == 2 {
//             NewId.push(*i);
//         }
//     }
//     let mut DelId = vec![];
//     for i in 0..AllTable.len() {
//         let mut ExtraMine = b;
//         for j in &TableId {
//             if AllTable[i][*j] == 1 {
//                 ExtraMine -= 1;
//             }
//         }
//         if ExtraMine < 0 || ExtraMine as usize > NewId.len() {
//             DelId.push(i);
//             continue;
//         }
//         let AddedTable = enumerateSub(NewId.len(), ExtraMine as usize);
//         for t in 0..NewId.len() {
//             AllTable[i][NewId[t]] = AddedTable[0][t];
//         }
//         for m in 1..AddedTable.len() {
//             AllTable.push(AllTable[i].clone());
//             for t in 0..NewId.len() {
//                 let len = AllTable.len() - 1;
//                 AllTable[len][NewId[t]] = AddedTable[m][t];
//             }
//         }
//     }
//     DelId.sort_by(|a, b| b.cmp(&a));
//     for i in DelId {
//         AllTable.remove(i);
//     }
//     AllTable
// }

fn cal_cell_and_equation_map(matrix_a: &Vec<Vec<i32>>) -> (Vec<Vec<usize>>, Vec<Vec<usize>>) {
    // cell_to_equation_map是方程中未知数的索引到方程的索引的映射
    // 方程中的未知数可以理解成未知的格子，每个方程可以理解成局面中的一个数字
    // 也可以理解成矩阵A的稀疏表示
    let cells_num = matrix_a[0].len();
    let equations_num = matrix_a.len();
    let mut cell_to_equation_map = vec![vec![]; cells_num];
    let mut equation_to_cell_map = vec![vec![]; equations_num];
    for i in 0..equations_num {
        for j in 0..cells_num {
            if matrix_a[i][j] >= 1 {
                equation_to_cell_map[i].push(j);
                cell_to_equation_map[j].push(i);
            }
        }
    }
    (cell_to_equation_map, equation_to_cell_map)
}

fn cal_table_minenum_recursion_step(
    idx: usize,
    current_amount: usize,
    table_minenum: &mut [Vec<usize>; 2],
    table_cell_minenum: &mut Vec<Vec<usize>>,
    // mut upper_limit: usize,
    // lower_limit: usize,
    matrixA_squeeze: &Vec<Vec<i32>>,
    matrix_b: &Vec<i32>,
    matrix_b_remain: &mut Vec<i32>,
    combination_relationship: &Vec<Vec<usize>>,
    cell_to_equation_map: &Vec<Vec<usize>>,
    equation_to_cell_map: &Vec<Vec<usize>>,
    mine_vec: &mut Vec<usize>,
) -> Result<bool, usize> {
    // mine_vec: 是雷位置都记录下来，只记录一个索引，可能有重复
    let cells_num = matrixA_squeeze[0].len();
    if idx >= cells_num {
        //终止条件
        let total_mines_num: usize = mine_vec.iter().sum();
        if total_mines_num >= table_minenum[1].len() {
            return Err(5);
        }
        table_minenum[1][total_mines_num] += current_amount;
        for (idn, n) in mine_vec.iter().enumerate() {
            table_cell_minenum[total_mines_num][idn] +=
                current_amount * n / combination_relationship[idn].len();
        }
        return Ok(true);
    }

    let mut upper_limit = combination_relationship[idx].len();
    let mut lower_limit = 0usize;
    for cell_i in &cell_to_equation_map[idx] {
        if matrixA_squeeze[*cell_i][idx] == 0 {
            continue;
        }
        let upper_limit_i = min(
            matrix_b_remain[*cell_i],
            combination_relationship[idx].len() as i32,
        );
        let mut lower_limit_i = matrix_b_remain[*cell_i];
        for j in &equation_to_cell_map[*cell_i] {
            if j > &idx {
                lower_limit_i -= combination_relationship[*j].len() as i32;
            }
        }
        if upper_limit_i < upper_limit as i32 {
            upper_limit = upper_limit_i as usize;
        }
        if lower_limit_i > lower_limit as i32 {
            lower_limit = lower_limit_i as usize;
        }
    }

    // println!("{:?}, {:?}", lower_limit, upper_limit + 1);
    // if lower_limit < upper_limit + 1 {
    for u in lower_limit..upper_limit + 1 {
        // let b = mine_vec[idx];
        mine_vec[idx] = u;
        if u > 0 {
            for tt in &cell_to_equation_map[idx] {
                matrix_b_remain[*tt] -= u as i32;
            }
        }
        let _ = cal_table_minenum_recursion_step(
            idx + 1,
            current_amount * C_query(combination_relationship[idx].len(), u),
            table_minenum,
            table_cell_minenum,
            &matrixA_squeeze,
            &matrix_b,
            matrix_b_remain,
            &combination_relationship,
            &cell_to_equation_map,
            &equation_to_cell_map,
            mine_vec,
        )?;
        for tt in &cell_to_equation_map[idx] {
            matrix_b_remain[*tt] += u as i32;
        }
        mine_vec[idx] = 0;
    }
    // }
    Ok(false)
}

pub fn cal_table_minenum_recursion(
    matrixA_squeeze: &Vec<Vec<i32>>,
    matrixx_squeeze: &Vec<(usize, usize)>,
    matrix_b: &Vec<i32>,
    combination_relationship: &Vec<Vec<usize>>,
) -> Result<([Vec<usize>; 2], Vec<Vec<usize>>), usize> {
    // 递归算法，得到雷数分布表和每格是雷情况数表，顺便计算最小、最大雷数
    // 输入矩阵必须是非空的，且行列数必须匹配
    // 行数和列数至少为1
    let cells_num = matrixx_squeeze.len();
    if cells_num > ENUM_LIMIT {
        // 超出枚举极限长度异常
        return Err(cells_num);
    }
    let cells_num_total = combination_relationship
        .iter()
        .fold(0, |item, x| item + x.len());
    // cells_num_total指合并前的格子数

    let mut flag_legal_board = true;
    let mut table_minenum: [Vec<usize>; 2] = [
        (0..cells_num_total + 1).collect::<Vec<usize>>(),
        vec![0; cells_num_total + 1],
    ];
    let (cell_to_equation_map, equation_to_cell_map) = cal_cell_and_equation_map(&matrixA_squeeze);
    // 计算两个映射表以减少复杂度
    // println!("cell_to_equation_map = {:?}; equation_to_cell_map = {:?}", cell_to_equation_map, equation_to_cell_map);

    let mut table_cell_minenum: Vec<Vec<usize>> = vec![vec![0; cells_num]; cells_num_total + 1];

    // println!("{:?}", matrixA_squeeze);
    cal_table_minenum_recursion_step(
        0,
        1,
        &mut table_minenum,
        &mut table_cell_minenum,
        &matrixA_squeeze,
        &matrix_b,
        &mut matrix_b.clone(),
        &combination_relationship,
        &cell_to_equation_map,
        &equation_to_cell_map,
        &mut (vec![0; cells_num]),
    )?;
    // println!("table_cell_minenum{:?}", table_cell_minenum);
    // println!("table_minenum{:?}", table_minenum);
    while table_minenum[1][0] == 0 {
        table_minenum[0].remove(0);
        table_minenum[1].remove(0);
        table_cell_minenum.remove(0);
        if table_cell_minenum.is_empty() {
            flag_legal_board = false;
            break;
        }
    }
    if flag_legal_board {
        while table_minenum[1][table_cell_minenum.len() - 1] == 0 {
            table_minenum[0].pop();
            table_minenum[1].pop();
            table_cell_minenum.pop();
        }
    }
    if flag_legal_board {
        Ok((table_minenum, table_cell_minenum))
    } else {
        return Err(1);
    }
}

// pub fn cal_table_minenum_enum(
//     matrixA_squeeze: &Vec<Vec<i32>>,
//     matrixx_squeeze: &Vec<(usize, usize)>,
//     matrix_b: &Vec<i32>,
//     combination_relationship: &Vec<Vec<usize>>,
// ) -> Result<([Vec<usize>; 2], Vec<Vec<usize>>), usize> {
//     // 拟弃用，用cal_table_minenum_recursion代替
//     // 枚举并统计，得到雷数分布表和每格是雷情况数表
//     let mut table_minenum: [Vec<usize>; 2] = [vec![], vec![]];
//     // 雷数分布表表：记录了每块（不包括内部块）每种总雷数下的是雷总情况数
//     // 例如：[[17, 18, 19, 20, 21, 22, 23, 24], [48, 2144, 16872, 49568, 68975, 48960, 16608, 2046]]
//     let mut table_cell_minenum: Vec<Vec<usize>> = vec![];
//     // 每格是雷情况数表：记录了每块每格（或者地位等同的复合格）、每种总雷数下的是雷情况数
//     if matrixx_squeeze.len() > 45 {
//         // 超出枚举极限长度
//         return Err(0);
//     }
//     let enum_comb_table: Vec<Vec<u8>> = enum_comb(&matrixA_squeeze, &matrixx_squeeze, &matrix_b);
//     if enum_comb_table.len() == 0 {
//         // 无解局面
//         return Err(1);
//     }
//     for s in enum_comb_table.clone() {
//         // println!("s: {:?}", s);
//         let s_sum = s.iter().sum::<u8>();
//         let mut si_num = 1; // 由于enum_comb_table中的格子每一个都代表了与其地位等同的所有格子，由此的情况数
//         for s_i in 0..s.len() {
//             si_num *= C_query(combination_relationship[s_i].len(), s[s_i]);
//         }
//         let fs = table_minenum[0]
//             .clone()
//             .iter()
//             .position(|x| *x == s_sum.into());
//         match fs {
//             None => {
//                 table_minenum[0].push(s_sum.into());
//                 table_minenum[1].push(si_num.into());
//                 let mut ss = vec![];
//                 for c in 0..s.len() {
//                     if s[c] == 0 {
//                         ss.push(0);
//                     } else {
//                         let mut sss = 1;
//                         for d in 0..s.len() {
//                             if c != d {
//                                 sss *= C_query(combination_relationship[d].len(), s[d]);
//                                 // println!("comb_relp_s = {:?}", comb_relp_s);
//                                 // println!("sss = {:?}", sss);
//                             } else {
//                                 sss *= C_query(combination_relationship[d].len() - 1, s[d] - 1);
//                             }
//                         }
//                         ss.push(sss as usize);
//                     }
//                 }
//                 table_cell_minenum.push(ss);
//             }
//             _ => {
//                 table_minenum[1][fs.unwrap()] += si_num as usize;
//                 for c in 0..s.len() {
//                     if s[c] == 0 {
//                         continue;
//                     } else {
//                         let mut sss = 1;
//                         for d in 0..s.len() {
//                             if c != d {
//                                 sss *= C_query(combination_relationship[d].len(), s[d]);
//                                 // println!("comb_relp_s=={:?}", comb_relp_s);
//                                 // println!("s=={:?}", s);
//                             } else {
//                                 sss *= C_query(combination_relationship[d].len() - 1, s[d] - 1);
//                             }
//                         }
//                         table_cell_minenum[fs.unwrap()][c] += sss as usize;
//                     }
//                 }
//             }
//         }
//     }

//     Ok((table_minenum, table_cell_minenum))
// }

/// 用几种模板，检测实际局面中是否有明显的死猜的结构。  
/// - 使用模板包括：工型、回型、器型。  
/// - 注意：对于一个局面，即使该检测返回true，也不能判断其必然是无猜的局面。想要真正判断一个局面无猜，请使用[is_solvable](#is_solvable)  
/// - 注意：局面至少大于4*4。
pub fn unsolvable_structure(BoardCheck: &Vec<Vec<i32>>) -> bool {
    let row = BoardCheck.len();
    let column = BoardCheck[0].len();
    let mut Board = vec![vec![0; column]; row];
    for i in 0..row {
        for j in 0..column {
            if BoardCheck[i][j] == -1 {
                Board[i][j] = -1;
            }
        }
    }
    for i in 0..row - 2 {
        // 检查左右两侧的工
        if i < row - 3 {
            if Board[i][0] == -1
                && Board[i][1] == -1
                && Board[i + 3][0] == -1
                && Board[i + 3][1] == -1
                && Board[i + 1][0] + Board[i + 2][0] == -1
                || Board[i][column - 1] == -1
                    && Board[i][column - 2] == -1
                    && Board[i + 3][column - 1] == -1
                    && Board[i + 3][column - 2] == -1
                    && Board[i + 1][column - 1] + Board[i + 2][column - 1] == -1
            {
                return true;
            }
        }
        if Board[i][2] == -1
            && Board[i + 1][2] == -1
            && Board[i + 2][2] == -1
            && Board[i + 1][0] + Board[i + 1][1] == -1
            || Board[i][column - 3] == -1
                && Board[i + 1][column - 3] == -1
                && Board[i + 2][column - 3] == -1
                && Board[i + 1][column - 1] + Board[i + 1][column - 2] == -1
            || Board[i][0] == -1
                && Board[i][1] == -1
                && Board[i + 1][1] == -1
                && Board[i + 2][1] == -1
                && Board[i + 2][0] == -1
                && Board[i + 1][0] == 0
            || Board[i][column - 1] == -1
                && Board[i][column - 2] == -1
                && Board[i + 1][column - 2] == -1
                && Board[i + 2][column - 2] == -1
                && Board[i + 2][column - 1] == -1
                && Board[i + 1][column - 1] == 0
        {
            return true;
        }
        if i < row - 3 {
            if Board[i][2] == -1
                && Board[i + 3][2] == -1
                && Board[i + 1][0] + Board[i + 1][1] == -1
                && Board[i + 1][1] + Board[i + 2][1] == -1
                && Board[i + 2][1] + Board[i + 2][0] == -1
                || Board[i][column - 3] == -1
                    && Board[i + 3][column - 3] == -1
                    && Board[i + 1][column - 1] + Board[i + 1][column - 2] == -1
                    && Board[i + 1][column - 2] + Board[i + 2][column - 2] == -1
                    && Board[i + 2][column - 2] + Board[i + 2][column - 1] == -1
            {
                return true;
            }
        }
    }
    for j in 0..column - 2 {
        // 检查上下两侧
        if j < column - 3 {
            if Board[0][j] == -1
                && Board[1][j] == -1
                && Board[0][j + 3] == -1
                && Board[1][j + 3] == -1
                && Board[0][j + 1] + Board[0][j + 2] == -1
                || Board[row - 1][j] == -1
                    && Board[row - 2][j] == -1
                    && Board[row - 1][j + 3] == -1
                    && Board[row - 2][j + 3] == -1
                    && Board[row - 1][j + 1] + Board[row - 1][j + 2] == -1
            {
                return true;
            }
        }
        if Board[2][j] == -1
            && Board[2][j + 1] == -1
            && Board[2][j + 2] == -1
            && Board[0][j + 1] + Board[1][j + 1] == -1
            || Board[row - 3][j] == -1
                && Board[row - 3][j + 1] == -1
                && Board[row - 3][j + 2] == -1
                && Board[row - 1][j + 1] + Board[row - 2][j + 1] == -1
            || Board[0][j] == -1
                && Board[1][j] == -1
                && Board[1][j + 1] == -1
                && Board[1][j + 2] == -1
                && Board[0][j + 2] == -1
                && Board[0][j + 1] == 0
            || Board[row - 1][j] == -1
                && Board[row - 2][j] == -1
                && Board[row - 2][j + 1] == -1
                && Board[row - 2][j + 2] == -1
                && Board[row - 1][j + 2] == -1
                && Board[row - 1][j + 1] == 0
        {
            return true;
        }
        if j < column - 3 {
            if Board[2][j] == -1
                && Board[2][j + 3] == -1
                && Board[0][j + 1] + Board[1][j + 1] == -1
                && Board[1][j + 1] + Board[1][j + 2] == -1
                && Board[1][j + 2] + Board[0][j + 2] == -1
                || Board[row - 3][j] == -1
                    && Board[row - 3][j + 3] == -1
                    && Board[row - 1][j + 1] + Board[row - 2][j + 1] == -1
                    && Board[row - 2][j + 1] + Board[row - 2][j + 2] == -1
                    && Board[row - 2][j + 2] + Board[row - 1][j + 2] == -1
            {
                return true;
            }
        }
    }
    if Board[0][2] == -1 && Board[1][2] == -1 && Board[0][0] + Board[0][1] == -1
        || Board[2][0] == -1 && Board[2][1] == -1 && Board[0][0] + Board[1][0] == -1
        || Board[0][column - 3] == -1
            && Board[1][column - 3] == -1
            && Board[0][column - 1] + Board[0][column - 2] == -1
        || Board[2][column - 1] == -1
            && Board[2][column - 2] == -1
            && Board[0][column - 1] + Board[1][column - 1] == -1
        || Board[row - 1][2] == -1
            && Board[row - 2][2] == -1
            && Board[row - 1][0] + Board[row - 1][1] == -1
        || Board[row - 3][0] == -1
            && Board[row - 3][1] == -1
            && Board[row - 1][0] + Board[row - 2][0] == -1
        || Board[row - 1][column - 3] == -1
            && Board[row - 2][column - 3] == -1
            && Board[row - 1][column - 1] + Board[row - 1][column - 2] == -1
        || Board[row - 3][column - 1] == -1
            && Board[row - 3][column - 2] == -1
            && Board[row - 1][column - 1] + Board[row - 2][column - 1] == -1
        || Board[0][1] + Board[1][1] + Board[1][0] == -3 && Board[0][0] == 0
        || Board[0][column - 2] + Board[1][column - 2] + Board[1][column - 1] == -3
            && Board[0][column - 1] == 0
        || Board[row - 1][column - 2] + Board[row - 2][column - 2] + Board[row - 2][column - 1]
            == -3
            && Board[row - 1][column - 1] == 0
        || Board[row - 1][1] + Board[row - 2][1] + Board[row - 2][0] == -3 && Board[row - 1][0] == 0
        || Board[2][2] == -1 && Board[0][1] + Board[1][1] == -1 && Board[1][0] + Board[1][1] == -1
        || Board[row - 3][2] == -1
            && Board[row - 1][1] + Board[row - 2][1] == -1
            && Board[row - 2][0] + Board[row - 2][1] == -1
        || Board[row - 3][column - 3] == -1
            && Board[row - 1][column - 2] + Board[row - 2][column - 2] == -1
            && Board[row - 2][column - 1] + Board[row - 2][column - 2] == -1
        || Board[2][column - 3] == -1
            && Board[0][column - 2] + Board[1][column - 2] == -1
            && Board[1][column - 1] + Board[1][column - 2] == -1
    {
        //检查四个角
        return true;
    }
    for i in 0..row - 2 {
        // 找中间的工、回、器形结构
        for j in 0..column - 2 {
            if j < column - 3 {
                if Board[i][j] == -1
                    && Board[i + 1][j] == -1
                    && Board[i + 2][j] == -1
                    && Board[i][j + 3] == -1
                    && Board[i + 1][j + 3] == -1
                    && Board[i + 2][j + 3] == -1
                    && Board[i + 1][j + 1] + Board[i + 1][j + 2] == -1
                {
                    return true;
                }
            }
            if i < row - 3 {
                if Board[i][j] == -1
                    && Board[i][j + 1] == -1
                    && Board[i][j + 2] == -1
                    && Board[i + 3][j] == -1
                    && Board[i + 3][j + 1] == -1
                    && Board[i + 3][j + 2] == -1
                    && Board[i + 1][j + 1] + Board[i + 2][j + 1] == -1
                {
                    return true;
                }
            }
            if Board[i][j] == -1
                && Board[i + 1][j] == -1
                && Board[i + 2][j] == -1
                && Board[i][j + 1] == -1
                && Board[i + 2][j + 1] == -1
                && Board[i][j + 2] == -1
                && Board[i + 1][j + 2] == -1
                && Board[i + 2][j + 2] == -1
                && Board[i + 1][j + 1] == 0
            {
                return true;
            }
            if j < column - 3 && i < row - 3 {
                if Board[i][j] == -1
                    && Board[i + 3][j] == -1
                    && Board[i][j + 3] == -1
                    && Board[i + 3][j + 3] == -1
                    && Board[i + 1][j + 1] + Board[i + 2][j + 1] == -1
                    && Board[i + 1][j + 1] + Board[i + 1][j + 2] == -1
                    && Board[i + 2][j + 1] + Board[i + 2][j + 2] == -1
                {
                    return true;
                }
            }
        }
    }
    false
}

// 专用于高级局面的3BV快速计算
pub fn cal_bbbv_exp(Board: &Vec<Vec<i32>>) -> usize {
    let mut board = Board.clone();
    let mut op_id = 0;
    let mut op_list = [false; 200];
    let mut bv = 0;
    for x in 0..16 {
        for y in 0..30 {
            if board[x][y] > 0 {
                board[x][y] = 1000000;
                bv += 1;
            } else if board[x][y] == 0 {
                let mut min_op_id = 1000;
                let mut flag_op = false; // 该空周围有无空的标志位
                if x >= 1 {
                    for j in max(1, y) - 1..min(30, y + 2) {
                        if board[x - 1][j] > 999999 {
                            board[x - 1][j] = 1;
                            bv -= 1;
                        } else if Board[x - 1][j] == 0 {
                            if board[x - 1][j] < min_op_id {
                                if flag_op {
                                    op_list[min_op_id as usize] = false;
                                } else {
                                    flag_op = true;
                                }
                                min_op_id = board[x - 1][j];
                            }
                        }
                    }
                }
                if y >= 1 {
                    if board[x][y - 1] > 999999 {
                        board[x][y - 1] = 1;
                        bv -= 1;
                    } else if Board[x][y - 1] == 0 {
                        if board[x][y - 1] < min_op_id {
                            if flag_op {
                                op_list[min_op_id as usize] = false;
                            } else {
                                flag_op = true;
                            }
                            min_op_id = board[x][y - 1];
                        }
                    }
                }
                if !flag_op {
                    op_id += 1;
                    op_list[op_id as usize] = true;
                }
            }
        }
    }
    for x in (0..16).rev() {
        for y in (0..30).rev() {
            if board[x][y] == 0 {
                if x <= 14 {
                    for j in max(1, y) - 1..min(30, y + 2) {
                        if board[x + 1][j] > 999999 {
                            board[x + 1][j] = 1;
                            bv -= 1;
                        } else if Board[x + 1][j] == 0 {
                            if board[x + 1][j] < board[x][y] {
                                op_list[board[x][y] as usize] = false;
                                board[x][y] = board[x + 1][j];
                            }
                        }
                    }
                }
                if y <= 28 {
                    if board[x][y + 1] > 999999 {
                        board[x][y + 1] = 1;
                        bv -= 1;
                    } else if Board[x][y + 1] == 0 {
                        if board[x][y + 1] < board[x][y] {
                            op_list[board[x][y] as usize] = false;
                            board[x][y] = board[x][y + 1];
                        }
                    }
                }
            }
        }
    }
    for i in 0..op_id + 1 {
        if op_list[i] {
            bv += 1;
        }
    }
    bv
}

// 把局面合法化：只能合法化简单的情况，不能应付所有的情况！因为检查一个局面是否合法也是NP难的
// 配合局面光学识别算法
// 局面中标记的标准是10为待判的雷，1到8，没有11、12
pub fn legalize_board(board: &mut Vec<Vec<i32>>) {
    let row = board.len();
    let column = board[0].len();
    for x in 0..row {
        for y in 0..column {
            if board[x][y] <= -1 || board[x][y] >= 13 || board[x][y] == 9 {
                // 把局面中明显未定义的数字改成未打开
                board[x][y] = 10;
            } else if board[x][y] >= 1 && board[x][y] <= 8 {
                let mut minenum_limit = 0;
                for i in max(1, x) - 1..min(row, x + 2) {
                    for j in max(1, y) - 1..min(column, y + 2) {
                        if board[i][j] == 10 || board[i][j] == 11 {
                            // 局面中的数字不能大于周围的未知格数
                            minenum_limit += 1;
                        }
                    }
                }
                board[x][y] = min(board[x][y], minenum_limit);
            }
        }
    }
}

// 重新分块矩阵
// 这些矩阵必须非空、没有空的块、没有b=0的情况
pub fn chunk_matrixes(
    matrix_as: &mut Vec<Vec<Vec<i32>>>,
    matrix_xs: &mut Vec<Vec<(usize, usize)>>,
    matrix_bs: &mut Vec<Vec<i32>>,
) {
    let block_num = matrix_bs.len();
    let mut aas = vec![];
    let mut xxs = vec![];
    let mut bbs = vec![];
    for _ in 0..block_num {
        let aa = matrix_as.pop().unwrap();
        let xx = matrix_xs.pop().unwrap();
        let bb = matrix_bs.pop().unwrap();
        let (mut a_, mut x_, mut b_) = chunk_matrix(aa, xx, bb);
        aas.append(&mut a_);
        xxs.append(&mut x_);
        bbs.append(&mut b_);
    }
    *matrix_as = aas;
    *matrix_xs = xxs;
    *matrix_bs = bbs;
}

// 重新分块一个矩阵
// 这些矩阵必须非空、没有空的块、没有b=0的情况
pub fn chunk_matrix(
    mut matrix_a: Vec<Vec<i32>>,
    mut matrix_x: Vec<(usize, usize)>,
    mut matrix_b: Vec<i32>,
) -> (Vec<Vec<Vec<i32>>>, Vec<Vec<(usize, usize)>>, Vec<Vec<i32>>) {
    let mut block_id = 0;
    let mut matrix_as = vec![];
    let mut matrix_xs = vec![];
    let mut matrix_bs = vec![];

    loop {
        let row_num = matrix_a.len();
        let column_num = matrix_a[0].len();
        let mut current_rows_bool = vec![false; row_num];
        let mut current_columns_bool = vec![false; column_num];
        current_columns_bool[0] = true;
        let mut column_buffer = vec![0];
        loop {
            let mut row_buffer = vec![];
            if column_buffer.is_empty() {
                break;
            }
            for i in &column_buffer {
                for idr in 0..matrix_a.len() {
                    if matrix_a[idr][*i] >= 1 && !current_rows_bool[idr] {
                        row_buffer.push(idr);
                        current_rows_bool[idr] = true;
                    }
                }
            }
            column_buffer.clear();
            if row_buffer.is_empty() {
                break;
            }
            for i in row_buffer {
                for (idc, &c) in matrix_a[i].iter().enumerate() {
                    if c >= 1 && !current_columns_bool[idc] {
                        column_buffer.push(idc);
                        current_columns_bool[idc] = true;
                    }
                }
            }
        }
        let mut current_rows = vec![];
        let mut current_columns = vec![];
        for (idx, &x) in current_columns_bool.iter().enumerate() {
            if x {
                current_columns.push(idx)
            }
        }
        for (idx, &x) in current_rows_bool.iter().enumerate() {
            if x {
                current_rows.push(idx)
            }
        }
        current_rows.sort_by(|a, b| b.cmp(a));
        current_rows.dedup();
        current_columns.sort_by(|a, b| b.cmp(a));
        current_columns.dedup();
        matrix_as.push(vec![vec![0; current_columns.len()]; current_rows.len()]);
        matrix_bs.push(vec![0; current_rows.len()]);
        matrix_xs.push(vec![(0, 0); current_columns.len()]);
        for (idx, x) in current_rows.iter().enumerate() {
            for (idy, y) in current_columns.iter().enumerate() {
                matrix_as[block_id][idx][idy] = matrix_a[*x][*y];
            }
        }
        for (idx, x) in current_rows.iter().enumerate() {
            matrix_bs[block_id][idx] = matrix_b[*x];
        }
        for (idy, y) in current_columns.iter().enumerate() {
            matrix_xs[block_id][idy] = matrix_x[*y];
        }
        for i in current_rows {
            matrix_a.remove(i);
            matrix_b.remove(i);
        }
        for j in current_columns {
            for k in 0..matrix_a.len() {
                matrix_a[k].remove(j);
            }
            matrix_x.remove(j);
        }

        if matrix_b.is_empty() {
            break;
        }
        block_id += 1;
    }
    (matrix_as, matrix_xs, matrix_bs)
}

#[test]
fn chunk_matrix_works() {
    let mut a = vec![
        vec![1, 1, 0, 0],
        vec![0, 0, 1, 1],
        vec![0, 1, 0, 0],
        vec![0, 0, 0, 1],
    ];
    let mut x = vec![(1, 2), (3, 4), (5, 6), (7, 8)];
    let mut b = vec![1, 2, 3, 4];
    let (aa, xx, bb) = chunk_matrix(a, x, b);
    println!("{:?}", xx);
}

// 找局面中间的格子的所在块的任意一个边界的格子。(可能不严格)
// 与弱无猜、准无猜有关。涉及判断是否为“必要的猜雷”。
// 点中间时，需要判断整块无猜以后，才能判定是合理的猜雷。
// xy处必须是10。第二个返回值true代表xy在边界，false代表在内部。
pub fn find_a_border_cell(
    board_of_game: &Vec<Vec<i32>>,
    xy: &(usize, usize),
) -> (Option<(usize, usize)>, bool) {
    let row = board_of_game.len();
    let column = board_of_game[0].len();
    for m in max(1, xy.0) - 1..min(row, xy.0 + 2) {
        for n in max(1, xy.1) - 1..min(column, xy.1 + 2) {
            if board_of_game[m][n] < 10 {
                return (Some(*xy), true);
            }
        }
    }
    let mut board_of_game_clone = board_of_game.clone();
    board_of_game_clone[xy.0][xy.1] = 100;
    let mut buffer = vec![(xy.0, xy.1)];
    while let Some(top) = buffer.pop() {
        let (i, j) = top;
        for m in max(1, i) - 1..min(row, i + 2) {
            for n in max(1, j) - 1..min(column, j + 2) {
                if (i != m || j != n) && board_of_game_clone[m][n] == 10 {
                    for mm in max(1, m) - 1..min(row, m + 2) {
                        for nn in max(1, n) - 1..min(column, n + 2) {
                            if board_of_game_clone[mm][nn] < 10 {
                                return (Some((m, n)), false);
                            }
                        }
                    }
                    buffer.push((m, n));
                    board_of_game_clone[m][n] = 100;
                }
            }
        }
    }
    (None, false)
}

/// 是局部最好的双击返回真，否则为假。方法是向四周试探一个位置，好的双击应该不能打开更多的格子。
/// - 不检查，但要保证pos位置处一定是合法、有效的双击，否则没意义。
/// - board_of_game必须是没有标错的雷的，如果分析录像，必须不是尸体。
pub fn is_good_chording(board_of_game: &Vec<Vec<i32>>, pos: (usize, usize)) -> bool {
    let row = board_of_game.len();
    let column = board_of_game[0].len();
    let mid_num = surround_cell_num(board_of_game, pos);
    if pos.0 > 0 {
        if mid_num < surround_cell_num(board_of_game, (pos.0 - 1, pos.1)) {
            return false;
        }
    }
    if pos.1 > 0 {
        if mid_num < surround_cell_num(board_of_game, (pos.0, pos.1 - 1)) {
            return false;
        }
    }
    if pos.0 + 1 < row {
        if mid_num < surround_cell_num(board_of_game, (pos.0 + 1, pos.1)) {
            return false;
        }
    }
    if pos.1 < column + 1 {
        if mid_num < surround_cell_num(board_of_game, (pos.0, pos.1 + 1)) {
            return false;
        }
    }
    return mid_num > 0;
}

// （双击位置）周围的格子（10）数，不合法则返回-1。
// - board_of_game必须是没有标错的雷的，如果分析录像，必须不是尸体。
fn surround_cell_num(board_of_game: &Vec<Vec<i32>>, pos: (usize, usize)) -> i8 {
    let row = board_of_game.len();
    let column = board_of_game[0].len();
    if board_of_game[pos.0][pos.1] > 8 || board_of_game[pos.0][pos.1] < 1 {
        return -1;
    }
    let mut flag_num = 0;
    let mut num = 0;
    for m in max(1, pos.0) - 1..min(row, pos.0 + 2) {
        for n in max(1, pos.1) - 1..min(column, pos.1 + 2) {
            if board_of_game[m][n] == 10 {
                num += 1;
            } else if board_of_game[m][n] == 11 {
                flag_num += 1;
            }
        }
    }
    if board_of_game[pos.0][pos.1] as i8 == flag_num {
        return num;
    } else {
        return -1;
    }
}

/// 算数字。局面上只有0和-1时，计算其他的数字。不具备幂等性！！！
pub fn cal_board_numbers(board: &mut Vec<Vec<i32>>) {
    let height = board.len();
    let width = board[0].len();
    for x in 0..height {
        for y in 0..width {
            if board[x][y] == -1 {
                for j in max(1, x) - 1..min(height, x + 2) {
                    for k in max(1, y) - 1..min(width, y + 2) {
                        if board[j][k] >= 0 {
                            board[j][k] += 1;
                        }
                    }
                }
            }
        }
    }
}
