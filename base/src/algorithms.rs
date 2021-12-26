
use crate::utils::{
    cal3BV, cal3BV_exp, cal_table_minenum_enum, cal_table_minenum_recursion, combine, enuOneStep,
    enum_comb, laymine_op_number, laymine_number, legalize_board, refreshBoard, refresh_matrix,
    refresh_matrixs, sum, unsolvable_structure, BigNumber, C_query, C,
};


#[cfg(any(feature = "py", feature = "rs"))]
use crate::OBR::ImageBoard;

use itertools::Itertools;

#[cfg(any(feature = "py", feature = "rs"))]
use rand::seq::SliceRandom;
#[cfg(any(feature = "py", feature = "rs"))]
use rand::thread_rng;

use std::cmp::{max, min};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

#[cfg(any(feature = "py", feature = "rs"))]
use tract_ndarray::Array;

#[cfg(any(feature = "py", feature = "rs"))]
use tract_onnx::prelude::*;

// 中高级的算法，例如无猜埋雷、判雷引擎、计算概率


/// 双集合判雷引擎，除判雷外，还会修改输入的局面，在局面上标是雷（11），但不标非雷（12）
pub fn solve_minus(
    MatrixA: &Vec<Vec<i32>>,
    Matrixx: &Vec<(usize, usize)>,
    Matrixb: &Vec<i32>,
    BoardofGame: &mut Vec<Vec<i32>>,
) -> (Vec<(usize, usize)>, bool) {
    let mut flag = false;
    let mut NotMine = vec![];
    let mut NotMineRel = vec![];
    let mut IsMineRel = vec![];
    let mut MatrixColumn = Matrixx.len();
    let mut MatrixRow = Matrixb.len();
    if MatrixRow <= 1 {
        return (NotMine, false);
    }
    for i in 1..MatrixRow {
        for j in 0..i {
            let mut ADval1 = vec![];
            let mut ADvaln1 = vec![];
            let mut FlagAdj = false;
            for k in 0..MatrixColumn {
                if MatrixA[i][k] >= 1 && MatrixA[j][k] >= 1 {
                    FlagAdj = true;
                    continue;
                }
                if MatrixA[i][k] - MatrixA[j][k] == 1 {
                    ADval1.push(k)
                } else if MatrixA[i][k] - MatrixA[j][k] == -1 {
                    ADvaln1.push(k)
                }
            }
            if FlagAdj {
                let bDval = Matrixb[i] - Matrixb[j];
                if ADval1.len() as i32 == bDval {
                    IsMineRel.append(&mut ADval1);
                    NotMineRel.append(&mut ADvaln1);
                } else if ADvaln1.len() as i32 == -bDval {
                    IsMineRel.append(&mut ADvaln1);
                    NotMineRel.append(&mut ADval1);
                }
            }
        }
    }
    if IsMineRel.len() > 0 || NotMineRel.len() > 0 {
        flag = true;
    }
    IsMineRel.dedup();
    NotMineRel.dedup();
    for i in 0..NotMineRel.len() {
        NotMine.push(Matrixx[NotMineRel[i]]);
    }
    for i in IsMineRel {
        BoardofGame[Matrixx[i].0][Matrixx[i].1] = 11;
    }
    (NotMine, flag)
}

/// 单集合判雷引擎，除判雷外，会在局面上标是雷（11），但不标非雷（12）
pub fn solve_direct(
    MatrixA: &mut Vec<Vec<i32>>,
    Matrixx: &mut Vec<(usize, usize)>,
    Matrixb: &mut Vec<i32>,
    BoardofGame: &mut Vec<Vec<i32>>,
) -> (Vec<(usize, usize)>, bool) {
    // 这三个矩阵被消费掉了比较可惜，待优化
    let mut flag = false;
    let mut NotMine = vec![];
    let mut MatrixColumn = Matrixx.len();
    let mut MatrixRow = Matrixb.len();
    for i in (0..MatrixRow).rev() {
        if sum(&MatrixA[i]) == Matrixb[i] {
            flag = true;
            for k in (0..MatrixColumn).rev() {
                if MatrixA[i][k] == 1 {
                    BoardofGame[Matrixx[k].0][Matrixx[k].1] = 11;
                    Matrixx.remove(k);
                    for t in 0..MatrixRow {
                        if MatrixA[t][k] == 0 {
                            MatrixA[t].remove(k);
                        } else {
                            MatrixA[t].remove(k);
                            Matrixb[t] -= 1;
                        }
                    }
                    MatrixColumn -= 1;
                }
            }
            MatrixA.remove(i);
            Matrixb.remove(i);
            MatrixRow -= 1;
        }
    }
    for i in 0..MatrixRow {
        if Matrixb[i] == 0 {
            flag = true;
            for k in 0..MatrixColumn {
                if MatrixA[i][k] == 1 {
                    NotMine.push(Matrixx[k]);
                }
            }
        }
    }
    NotMine.dedup(); // 去重
                     // for i in &NotMine {
                     //     BoardofGame[i.0][i.1] = 12;
                     // }
    (NotMine, flag)
}

/// 输入局面、未被标出的雷数  
/// 局面中可以标雷（11），但必须全部标对  
/// 未知雷数 = 总雷数 - 已经标出的雷  
/// 若超出枚举长度，返回空向量  
/// 返回所有边缘格子是雷的概率、内部未知格子是雷的概率、局面中总未知雷数的范围  
/// 若没有内部未知区域，返回NaN
pub fn cal_possibility(
    board_of_game: &Vec<Vec<i32>>,
    mut mine_num: f64,
) -> Result<(Vec<((usize, usize), f64)>, f64, [usize; 3]), usize> {
    let mut p = vec![];
    let mut table_cell_minenum_s: Vec<Vec<Vec<usize>>> = vec![];
    // 每块每格雷数表：记录了每块每格（或者地位等同的复合格）、每种总雷数下的是雷情况数
    let mut comb_relp_s = vec![]; // 记录了方格的组合关系
                                  // let mut enum_comb_table_s = vec![];
    let mut table_minenum_s: Vec<[Vec<usize>; 2]> = vec![];
    // 每块雷数分布表：记录了每块（不包括内部块）每种总雷数下的是雷总情况数
    // 例如：[[[17, 18, 19, 20, 21, 22, 23, 24], [48, 2144, 16872, 49568, 68975, 48960, 16608, 2046]]]
    let (matrix_a_s, matrix_x_s, matrix_b_s, unknow_block, is_mine_num) =
        refresh_matrixs(&board_of_game);
    let block_num = matrix_a_s.len(); // 整个局面被分成的块数

    let mut matrixA_squeeze_s: Vec<Vec<Vec<i32>>> = vec![];
    let mut matrixx_squeeze_s: Vec<Vec<(usize, usize)>> = vec![];
    // let mut min_max_mine_num = [0, 0];
    for i in 0..block_num {
        let (matrixA_squeeze, matrixx_squeeze, combination_relationship) =
            combine(&matrix_a_s[i], &matrix_x_s[i]);
        comb_relp_s.push(combination_relationship);
        matrixA_squeeze_s.push(matrixA_squeeze);
        matrixx_squeeze_s.push(matrixx_squeeze);
    }
    // 分块枚举后，根据雷数限制，删除某些情况
    for i in 0..block_num {
        let table_minenum_i;
        let table_cell_minenum_i;
        match cal_table_minenum_recursion(
            &matrixA_squeeze_s[i],
            &matrixx_squeeze_s[i],
            &matrix_b_s[i],
            &comb_relp_s[i],
        ) {
            Ok((table_minenum_i_, table_cell_minenum_i_)) => {
                table_minenum_i = table_minenum_i_;
                table_cell_minenum_i = table_cell_minenum_i_;
            }
            Err(e) => return Err(e),
        };
        // min_max_mine_num[0] += table_minenum_i[0][0];
        // min_max_mine_num[1] += table_minenum_i[0][table_minenum_i[0].len() - 1];

        table_cell_minenum_s.push(table_cell_minenum_i);
        table_minenum_s.push(table_minenum_i);
    } // 第一步，整理出每块每格雷数情况表、每块雷数分布表、每块雷分布情况总数表
    let mut min_mine_num = 0;
    let mut max_mine_num = 0;
    for i in 0..block_num {
        min_mine_num += table_minenum_s[i][0].iter().min().unwrap();
        max_mine_num += table_minenum_s[i][0].iter().max().unwrap();
    }
    let mine_num = if mine_num <= 1.0 {
        let mn = ((board_of_game.len() * board_of_game[0].len()) as f64 * mine_num) as usize;
        min(
            max(mn - is_mine_num, min_mine_num),
            max_mine_num + unknow_block,
        )
    } else {
        mine_num as usize - is_mine_num
    };

    max_mine_num = min(max_mine_num, mine_num);
    let unknow_mine_num: Vec<usize> =
        (mine_num - max_mine_num..min(mine_num - min_mine_num, unknow_block) + 1).collect();
    // 这里的写法存在极小的风险，例如边缘格雷数分布是0，1，3，而我们直接认为了可能有2
    let mut unknow_mine_s_num = vec![];
    for i in &unknow_mine_num {
        unknow_mine_s_num.push(C(unknow_block, *i));
    }
    // 第二步，整理内部未知块雷数分布表，并筛选。这样内部未知雷块和边缘雷块的地位视为几乎等同，但数据结构不同
    table_minenum_s.push([unknow_mine_num.clone(), vec![]]);
    // 这里暂时不知道怎么写，目前这样写浪费了几个字节的内存
    // 未知区域的情况数随雷数的分布不能存在表table_minenum中，因为格式不一样，后者是大数类型
    let mut mine_in_each_block = (0..block_num + 1)
        .map(|i| 0..table_minenum_s[i][0].len())
        .multi_cartesian_product()
        .collect::<Vec<_>>();
    for i in (0..mine_in_each_block.len()).rev() {
        let mut total_num = 0;
        for j in 0..block_num + 1 {
            total_num += table_minenum_s[j][0][mine_in_each_block[i][j]];
        }
        if total_num != mine_num {
            mine_in_each_block.remove(i);
        }
    }
    // println!("mine_in_each_block: {:?}", mine_in_each_block);
    // 第三步，枚举每块雷数情况索引表：行代表每种情况，列代表每块雷数的索引，最后一列代表未知区域雷数
    let mut table_minenum_other: Vec<Vec<BigNumber>> = vec![];
    for i in 0..block_num + 1 {
        table_minenum_other.push(vec![
            BigNumber { a: 0.0, b: 0 };
            table_minenum_s[i][0].len()
        ]);
    } // 初始化
    for s in mine_in_each_block {
        for i in 0..block_num {
            let mut s_num = BigNumber { a: 1.0, b: 0 };
            let mut s_mn = mine_num; // 未知区域中的雷数
            for j in 0..block_num {
                if i != j {
                    s_num.mul_usize(table_minenum_s[j][1][s[j]]);
                }
                s_mn -= table_minenum_s[j][0][s[j]];
            }
            let ps = unknow_mine_num.iter().position(|x| *x == s_mn).unwrap();
            s_num.mul_big_number(&unknow_mine_s_num[ps]);
            table_minenum_other[i][s[i]].add_big_number(&s_num);
        }
        let mut s_num = BigNumber { a: 1.0, b: 0 };
        let mut s_mn = mine_num; // 未知区域中的雷数
        for j in 0..block_num {
            s_num.mul_usize(table_minenum_s[j][1][s[j]]);
            s_mn -= table_minenum_s[j][0][s[j]];
        }
        let ps = unknow_mine_num.iter().position(|x| *x == s_mn).unwrap();
        table_minenum_other[block_num][ps].add_big_number(&s_num);
    }
    // 第四步，计算每块其他雷数情况表
    let mut T = BigNumber { a: 0.0, b: 0 };
    for i in 0..unknow_mine_s_num.len() {
        let mut t = table_minenum_other[block_num][i].clone();
        t.mul_big_number(&unknow_mine_s_num[i]);
        T.add_big_number(&t);
    }
    // 第五步，计算局面总情况数

    for i in 0..block_num {
        for cells_id in 0..comb_relp_s[i].len() {
            let cells_len = comb_relp_s[i][cells_id].len();
            for cell_id in 0..cells_len {
                let mut s_cell = BigNumber { a: 0.0, b: 0 };
                for s in 0..table_minenum_other[i].len() {
                    let mut o = table_minenum_other[i][s].clone();
                    o.mul_usize(table_cell_minenum_s[i][s][cells_id]);
                    s_cell.add_big_number(&o);
                }
                let p_cell = s_cell.div_big_num(&T);
                let id = comb_relp_s[i][cells_id][cell_id];
                p.push((matrix_x_s[i][id], p_cell));
            }
        }
    }
    // 第六步，计算边缘每格是雷的概率
    let mut u_s = BigNumber { a: 0.0, b: 0 };
    for i in 0..unknow_mine_num.len() {
        let mut u = table_minenum_other[block_num][i].clone();
        u.mul_big_number(&unknow_mine_s_num[i]);
        u.mul_usize(unknow_mine_num[i]);
        u_s.add_big_number(&u);
    }
    let p_unknow = u_s.div_big_num(&T) / unknow_block as f64;
    // 第七步，计算内部未知区域是雷的概率

    Ok((
        p,
        p_unknow,
        [
            min_mine_num + is_mine_num,
            mine_num + is_mine_num,
            max_mine_num + is_mine_num + unknow_block,
        ],
    ))
}

/// 计算局面中各位置是雷的概率，按照所在的位置返回
pub fn cal_possibility_onboard(
    board_of_game: &Vec<Vec<i32>>,
    mine_num: f64,
) -> Result<(Vec<Vec<f64>>, [usize; 3]), usize> {
    let mut p = vec![vec![-1.0; board_of_game[0].len()]; board_of_game.len()];
    let pp;
    match cal_possibility(&board_of_game, mine_num) {
        Ok(ppp) => pp = ppp,
        Err(e) => return Err(e),
    }
    for i in pp.0 {
        p[i.0 .0][i.0 .1] = i.1;
    }
    for r in 0..board_of_game.len() {
        for c in 0..board_of_game[0].len() {
            if board_of_game[r][c] == 11 {
                p[r][c] = 1.0;
            } else if board_of_game[r][c] == 10 && p[r][c] < -0.5 {
                p[r][c] = pp.1;
            } else if board_of_game[r][c] == 12 {
                p[r][c] = 0.0;
            }
        }
    }
    Ok((p, pp.2))
}

/// 计算单点开空概率  
/// 输入局面、未被标出的雷数、坐标  
/// 返回坐标处开空的概率  
/// # Example
/// ```
/// let game_board = vec![
///     vec![10, 10,  1,  1, 10,  1,  0,  0],
///     vec![10, 10,  1, 10, 10,  3,  2,  1],
///     vec![10, 10, 10, 10, 10, 10, 10, 10],
///     vec![10, 10, 10, 10, 10, 10, 10, 10],
///     vec![10, 10, 10, 10, 10, 10, 10, 10],
///     vec![10, 10, 10, 10,  2, 10, 10, 10],
///     vec![10, 10, 10, 10, 10, 10, 10, 10],
///     vec![10, 10, 10, 10, 10, 10, 10, 10],
/// ];
/// let ans = cal_is_op_possibility_cells(&game_board, 20.0, &vec![[0, 0], [1, 1], [1, 6], [7, 2]]);
/// print!("{:?}", ans)
/// ```
pub fn cal_is_op_possibility_cells(
    board_of_game: &Vec<Vec<i32>>,
    mine_num: f64,
    cells: &Vec<[usize; 2]>,
) -> Vec<f64> {
    let mut poss = vec![1.0; cells.len()];
    let row = board_of_game.len();
    let column = board_of_game[0].len();
    for (cell_id, cell) in cells.iter().enumerate() {
        let mut board_of_game_modified = board_of_game.clone();
        'outer: for m in max(1, cell[0]) - 1..min(row, cell[0] + 2) {
            for n in max(1, cell[1]) - 1..min(column, cell[1] + 2) {
                if (board_of_game[m][n] < 10 && m == cell[0] && n == cell[1])
                    || board_of_game[m][n] == 11
                {
                    poss[cell_id] = 0.0;
                    break 'outer;
                } else if board_of_game[m][n] == 12 || board_of_game[m][n] < 10 {
                    continue;
                } else {
                    let p;
                    match cal_possibility_onboard(&board_of_game_modified, mine_num) {
                        Ok((ppp, _)) => p = ppp,
                        Err(e) => {
                            poss[cell_id] = 0.0;
                            break 'outer;
                        }
                    };
                    poss[cell_id] *= 1.0 - p[m][n];
                    board_of_game_modified[m][n] = 12;
                }
            }
        }
    }
    poss
}

/// 埋雷，使得起手位置必为空  
/// 输入行数、列数、雷数、起手位置行数、起手位置列数、最小3BV、最大3BV、最大尝试次数、模式（保留）  
/// 输出局面和参数，分别为：是否满足3BV上下限、3BV、尝试次数  
/// # Example
/// ```
/// layMineOp(16, 30, 99, 0, 0, 100, 381, 10000, 0)
/// ```
pub fn laymine_op(
    row: usize,
    column: usize,
    MineNum: usize,
    x0: usize,
    y0: usize,
    Min3BV: usize,
    Max3BV: usize,
    MaxTimes: usize,
    method: usize,
) -> (Vec<Vec<i32>>, Vec<usize>) {
    let mut times = 0;
    let mut Parameters = vec![];
    let mut Num3BV = 0;
    let mut Board = vec![vec![0; column]; row];
    while times < MaxTimes {
        Board = laymine_op_number(row, column, MineNum, x0, y0);
        times += 1;
        let mut Num3BV = cal3BV(&Board);
        if Num3BV >= Min3BV && Num3BV <= Max3BV {
            Parameters.push(1);
            Parameters.push(Num3BV);
            Parameters.push(times);
            return (Board, Parameters);
        }
    }
    Parameters.push(0);
    Parameters.push(Num3BV);
    Parameters.push(times);
    (Board, Parameters)
}

/// 枚举法判雷引擎  
/// 输入分块好的矩阵、局面、枚举长度限制  
/// 输出不是雷的位置  
/// 注意：不修改输入进来的局面，即不帮助标雷
pub fn solve_enumerate(
    matrix_as: &Vec<Vec<Vec<i32>>>,
    matrix_xs: &Vec<Vec<(usize, usize)>>,
    matrix_bs: &Vec<Vec<i32>>,
    board_of_game: &Vec<Vec<i32>>,
    enuLimit: usize,
) -> (Vec<(usize, usize)>, bool) {
    let mut flag = false;
    let mut NotMine = vec![];
    let block_num = matrix_xs.len();
    if block_num > enuLimit {
        return (vec![], false);
    }

    let mut comb_relp_s = vec![];
    let mut matrixA_squeeze_s: Vec<Vec<Vec<i32>>> = vec![];
    let mut matrixx_squeeze_s: Vec<Vec<(usize, usize)>> = vec![];
    for i in 0..block_num {
        let (matrixA_squeeze, matrixx_squeeze, combination_relationship) =
            combine(&matrix_as[i], &matrix_xs[i]);
        comb_relp_s.push(combination_relationship);
        matrixA_squeeze_s.push(matrixA_squeeze);
        matrixx_squeeze_s.push(matrixx_squeeze);
    }
    for i in 0..block_num {
        let (table_minenum_i, table_cell_minenum_i) = cal_table_minenum_recursion(
            &matrixA_squeeze_s[i],
            &matrixx_squeeze_s[i],
            &matrix_bs[i],
            &comb_relp_s[i],
        )
        .unwrap();
        'outer: for jj in 0..table_cell_minenum_i[0].len() {
            for ii in 0..table_cell_minenum_i.len() {
                if table_cell_minenum_i[ii][jj] > 0 {
                    continue 'outer;
                }
            }
            for kk in &comb_relp_s[i][jj] {
                NotMine.push(matrix_xs[i][*kk]);
            }
        }
    }
    (NotMine, flag)
}

// 判断当前是否获胜
// 游戏局面中必须没有标错的雷
// 这个函数不具备普遍意义
fn isVictory(BoardofGame: &Vec<Vec<i32>>, Board: &Vec<Vec<i32>>) -> bool {
    let row = BoardofGame.len();
    let col = BoardofGame[0].len();
    for i in 0..row {
        for j in 0..col {
            if BoardofGame[i][j] == 10 && Board[i][j] != -1 {
                return false;
            }
        }
    }
    return true;
}

/// 从指定位置开始扫，判断局面是否无猜  
/// 注意：周围一圈都是雷，那么若中间是雷不算猜，若中间不是雷算猜
pub fn is_solvable(Board: &Vec<Vec<i32>>, x0: usize, y0: usize, enuLimit: usize) -> bool {
    if unsolvable_structure(&Board) {
        //若包含不可判雷结构，则不是无猜
        return false;
    }
    // println!("Board: {:?}", Board);
    let row = Board.len();
    let column = Board[0].len();
    let mut BoardofGame = vec![vec![10; column]; row];
    // 10是未打开，11是标雷
    // 局面大小必须超过6*6
    refreshBoard(&Board, &mut BoardofGame, vec![(x0, y0)]);
    if isVictory(&BoardofGame, &Board) {
        return true; // 暂且认为点一下就扫开也是可以的
    }
    let mut NotMine;
    let mut flag;
    loop {
        let (mut MatrixA, mut Matrixx, mut Matrixb) = refresh_matrix(&BoardofGame);
        let ans = solve_direct(&mut MatrixA, &mut Matrixx, &mut Matrixb, &mut BoardofGame);
        NotMine = ans.0;
        flag = ans.1;
        if !flag {
            let ans = solve_minus(&MatrixA, &Matrixx, &Matrixb, &mut BoardofGame);
            NotMine = ans.0;
            flag = ans.1;
            if !flag {
                let (mut Matrix_as, mut Matrix_xs, mut Matrix_bs, _, _) =
                    refresh_matrixs(&BoardofGame);
                // print!("BoardofGame: {:?}", BoardofGame);
                let ans = solve_enumerate(
                    &Matrix_as,
                    &Matrix_xs,
                    &Matrix_bs,
                    &mut BoardofGame,
                    enuLimit,
                );
                NotMine = ans.0;
                flag = ans.1;
                if !flag {
                    return false;
                }
            }
        }
        refreshBoard(&Board, &mut BoardofGame, NotMine);
        if isVictory(&BoardofGame, &Board) {
            return true;
        }
    }
}

/// 删选法多（10）线程无猜埋雷  
/// 3BV下限、上限，最大尝试次数，返回是否成功  
/// 若不成功返回最后生成的局面，此时则不一定无猜
#[cfg(any(feature = "py", feature = "rs"))]
pub fn laymine_solvable_thread(
    row: usize,
    column: usize,
    MineNum: usize,
    x0: usize,
    y0: usize,
    Min3BV: usize,
    Max3BV: usize,
    mut MaxTimes: usize,
    enuLimit: usize,
) -> (Vec<Vec<i32>>, [usize; 3]) {
    let mut parameters = [0, 0, 0];
    let mut game_board = vec![vec![0; column]; row];
    let mut handles = vec![];
    let flag_exit = Arc::new(Mutex::new(0));
    let (tx, rx) = mpsc::channel(); // mpsc 是多个发送者，一个接收者
    for ii in (1..11).rev() {
        let tx_ = mpsc::Sender::clone(&tx);
        let max_time = MaxTimes / ii;
        MaxTimes -= max_time;
        let flag_exit = Arc::clone(&flag_exit);
        let handle = thread::spawn(move || {
            let mut Num3BV;
            let mut counter = 0;
            let mut Board = vec![vec![0; column]; row];
            let mut para = [0, 0, 0];
            while counter < max_time {
                {
                    let f = flag_exit.lock().unwrap();
                    if *f == 1 {
                        break;
                    }
                } // 这块用花括号控制生命周期
                let Board_ = laymine_op_number(row, column, MineNum, x0, y0);
                counter += 1;
                if is_solvable(&Board_, x0, y0, enuLimit) {
                    Num3BV = cal3BV(&Board_);
                    if Num3BV >= Min3BV && Num3BV <= Max3BV {
                        para[0] = 1;
                        para[1] = Num3BV;
                        para[2] = counter;
                        for i in 0..row {
                            for j in 0..column {
                                Board[i][j] = Board_[i][j];
                            }
                        }
                        let mut f = flag_exit.lock().unwrap();
                        *f = 1;
                        tx_.send((Board, para)).unwrap();
                        break;
                    }
                }
            }
            let Board_ = laymine_op_number(row, column, MineNum, x0, y0);
            Num3BV = cal3BV(&Board_);
            para[0] = 0;
            para[1] = Num3BV;
            para[2] = max_time + 1;
            tx_.send((Board_, para)).unwrap();
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
    let received = rx.recv().unwrap(); // 尝试次数仅仅为单个线程的次数，并不准
    parameters[0] = received.1[0];
    parameters[1] = received.1[1];
    parameters[2] = received.1[2];
    for i in 0..row {
        for j in 0..column {
            game_board[i][j] = received.0[i][j];
        }
    }
    (game_board, parameters)
}

/// 删选法单线程无猜埋雷  
/// 3BV下限、上限，最大尝试次数，返回是否成功  
/// 若不成功返回最后生成的局面，此时则不一定无猜
pub fn laymine_solvable(
    row: usize,
    column: usize,
    MineNum: usize,
    x0: usize,
    y0: usize,
    Min3BV: usize,
    Max3BV: usize,
    MaxTimes: usize,
    enuLimit: usize,
) -> (Vec<Vec<i32>>, Vec<usize>) {
    let mut times = 0;
    let mut Parameters = vec![];
    let mut Board;
    let mut Num3BV;
    while times < MaxTimes {
        Board = laymine_op_number(row, column, MineNum, x0, y0);
        times += 1;
        if is_solvable(&Board, x0, y0, enuLimit) {
            Num3BV = cal3BV(&Board);
            if Num3BV >= Min3BV && Num3BV <= Max3BV {
                Parameters.push(1);
                Parameters.push(Num3BV);
                Parameters.push(times);
                return (Board, Parameters);
            }
        }
    }
    Board = laymine_op_number(row, column, MineNum, x0, y0);
    Num3BV = cal3BV(&Board);
    Parameters.push(0);
    Parameters.push(Num3BV);
    Parameters.push(times);
    (Board, Parameters)
}

/// 调整法无猜埋雷（没写好）
pub fn laymine_solvable_adjust(
    row: usize,
    column: usize,
    mine_num: usize,
    x0: usize,
    y0: usize,
    min_3BV: usize,
    max_3BV: usize,
    max_times: usize,
    enu_limit: usize,
) -> (Vec<Vec<i32>>, [usize; 3]) {
    // 利用局面调整算法，无猜埋雷
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
    if row * column - area_op < mine_num {
        // 雷数太多以致起手无法开空，此时放弃无猜，返回任意一种局面
        let t = laymine(
            row, column, mine_num, x0, y0, min_3BV, max_3BV, max_times, 0,
        );
        return (t.0, [0, t.1[1], t.1[2]]);
    }
    let remain_mine_num = mine_num;
    let remain_inside_cells_num = row * column - area_op;
    for time in 0..max_times {}
    (vec![], [0, 0, 0])
}

/// 标准埋雷，参数依次是行、列、雷数、起手位置的第几行、第几列  
/// 适用于游戏的埋雷算法  
/// 起手不开空，必不为雷  
/// 返回二维列表，0~8代表数字，-1代表雷  
/// method = 0筛选算法；1调整算法（保留）
pub fn laymine(
    row: usize,
    column: usize,
    MineNum: usize,
    x0: usize,
    y0: usize,
    Min3BV: usize,
    Max3BV: usize,
    MaxTimes: usize,
    method: usize,
) -> (Vec<Vec<i32>>, Vec<usize>) {
    
    let mut times = 0;
    let mut Parameters = vec![];
    let mut Num3BV = 0;
    let mut Board = vec![vec![0; column]; row];
    while times < MaxTimes {
        Board = laymine_number(row, column, MineNum, x0, y0);
        times += 1;
        Num3BV = cal3BV(&Board);
        if Num3BV >= Min3BV && Num3BV <= Max3BV {
            Parameters.push(1);
            Parameters.push(Num3BV);
            Parameters.push(times);
            return (Board, Parameters);
        }
    }
    Parameters.push(0);
    Parameters.push(Num3BV);
    Parameters.push(times);
    (Board, Parameters)
}

#[cfg(any(feature = "py", feature = "rs"))]
pub fn sample_3BVs_exp(x0: usize, y0: usize, n: usize) -> [usize; 382] {
    // 从标准高级中采样计算3BV
    // 16线程计算
    let n0 = n / 16;
    let mut threads = vec![];
    for i in 0..16 {
        let join_item =
            thread::spawn(move || -> [usize; 382] { laymine_number_study_exp(x0, y0, n0) });
        threads.push(join_item);
    }
    let mut aa = [0; 382];
    for i in threads.into_iter().map(|c| c.join().unwrap()) {
        for ii in 0..382 {
            aa[ii] += i[ii];
        }
    }
    aa
}

#[cfg(any(feature = "py", feature = "rs"))]
fn laymine_number_study_exp(x0: usize, y0: usize, n: usize) -> [usize; 382] {
    // 专用埋雷并计算3BV引擎，用于研究
    let mut rng = thread_rng();
    // let area: usize = 16 * 30 - 1;
    let pointer = x0 + y0 * 16;
    let mut bv_record = [0; 382];
    for id in 0..n {
        let mut Board1Dim = [0; 479];
        for i in 380..479 {
            Board1Dim[i] = -1;
        }

        Board1Dim.shuffle(&mut rng);
        let mut Board1Dim_2 = [0; 480];
        // Board1Dim_2.reserve(area + 1);

        for i in 0..pointer {
            Board1Dim_2[i] = Board1Dim[i];
        }
        Board1Dim_2[pointer] = 0;
        for i in pointer..479 {
            Board1Dim_2[i + 1] = Board1Dim[i];
        }
        let mut Board: Vec<Vec<i32>> = vec![vec![0; 30]; 16];
        for i in 0..480 {
            if Board1Dim_2[i] < 0 {
                let x = i % 16;
                let y = i / 16;
                Board[x][y] = -1;
                for j in max(1, x) - 1..min(16, x + 2) {
                    for k in max(1, y) - 1..min(30, y + 2) {
                        if Board[j][k] >= 0 {
                            Board[j][k] += 1;
                        }
                    }
                }
            }
        }
        bv_record[cal3BV_exp(&Board)] += 1;
    }
    bv_record
}

#[cfg(any(feature = "py", feature = "rs"))]
fn OBR_cell(
    cell_image: &Vec<f32>,
    model: &tract_onnx::prelude::SimplePlan<
        tract_onnx::prelude::TypedFact,
        std::boxed::Box<dyn tract_onnx::prelude::TypedOp>,
        tract_onnx::prelude::Graph<
            tract_onnx::prelude::TypedFact,
            std::boxed::Box<dyn tract_onnx::prelude::TypedOp>,
        >,
    >,
) -> TractResult<i32> {
    // 光学识别单个cell

    let image: Tensor = Array::from_shape_vec((1, 3, 16, 16), (*cell_image).clone())
        .unwrap()
        .into();
    let result = model.run(tvec!(image))?;

    let best = result[0]
        .to_array_view::<f32>()?
        .iter()
        .cloned()
        .zip(1..)
        .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    match best.unwrap().1 {
        1 => Ok(0),
        2 => Ok(1),
        3 => Ok(2),
        4 => Ok(3),
        5 => Ok(4),
        6 => Ok(5),
        7 => Ok(6),
        8 => Ok(7),
        9 => Ok(8),
        10 => Ok(10),
        _ => Ok(11),
    }
}

/// 输入列向量形式的三通道的像素数据，图像的高度、宽度；  
/// 以下提供一段用python调用时的demo  
/// ```python
/// import ms_toollib
/// import matplotlib.image as mpimg
/// import numpy as np
/// file = r'C:\Users\jia32\Desktop\无标题.png'# 彩色图片
/// lena = mpimg.imread(file)
/// (height, width) = lena.shape[:2]
/// lena = np.concatenate((lena, 255.0 * np.ones((height, width, 1))), 2)
/// lena = (np.reshape(lena, -1) * 255).astype(np.uint32)
/// board = ms_toollib.py_OBR_board(lena, height, width)
/// print(np.array(board))# 打印识别出的局面
/// poss = ms_toollib.py_cal_possibility(board, 99)
/// print(poss)# 用雷的总数计算概率
/// poss_onboard = ms_toollib.py_cal_possibility_onboard(board, 99)
/// print(poss_onboard)# 用雷的总数计算概率，输出局面对应的位置
/// poss_ = ms_toollib.py_cal_possibility_onboard(board, 0.20625)
/// print(poss_)# 用雷的密度计算概率
/// ```
/// 注意：必须配合"params.onnx"参数文件调用  
/// 注意：由于利用了神经网络技术，可能发生识别错误，此时输出是不一定合法的局面
#[cfg(any(feature = "py", feature = "rs"))]
pub fn OBR_board(
    data_vec: Vec<usize>,
    height: usize,
    width: usize,
) -> Result<Vec<Vec<i32>>, String> {
    // 为什么输入形式这么奇怪呢？主要是为了适配python截图出来的原始数据格式
    if height <= 24 || width <= 24 {
        return Err("one input size of the board is smaller than 3".to_string());
    }
    let mut image_board = ImageBoard::new(data_vec, height, width);
    image_board.get_pos_pixel();
    if image_board.r <= 3 || image_board.c <= 3 {
        return Err("one size of the board seems to be smaller than 3".to_string());
    }
    let mut board = vec![vec![0i32; image_board.c]; image_board.r];
    let model = (tract_onnx::onnx()
        .model_for_path("params.onnx")
        .unwrap()
        .with_input_fact(
            0,
            InferenceFact::dt_shape(f32::datum_type(), tvec!(1, 3, 16, 16)),
        )
        .unwrap()
        .into_optimized()
        .unwrap()
        .into_runnable())
    .unwrap();
    for i in 0..image_board.r {
        for j in 0..image_board.c {
            let cell = OBR_cell(&image_board.extra_save_cell(i, j, 16), &model).unwrap();
            board[i][j] = cell;
        }
    }
    legalize_board(&mut board);
    Ok(board)
}

/// 对局面用单集合、双集合判雷引擎，快速标雷、标非雷，以供概率计算引擎处理  
/// 相当于一种预处理，即先标出容易计算的
pub fn mark_board(board: &mut Vec<Vec<i32>>) {
    
    let (mut matrix_a, mut matrix_x, mut matrix_b) = refresh_matrix(&board);
    let ans = solve_direct(&mut matrix_a, &mut matrix_x, &mut matrix_b, board);
    let not_mine = ans.0;
    for i in not_mine {
        board[i.0][i.1] = 12;
    }
    let (matrix_a, matrix_x, matrix_b) = refresh_matrix(&board);
    let ans = solve_minus(&matrix_a, &matrix_x, &matrix_b, board);
    let not_mine = ans.0;
    for i in not_mine {
        board[i.0][i.1] = 12;
    }
}

/// 求出所有非雷的位置
pub fn solve_all_notmine_on_board(board_: &Vec<Vec<i32>>, enuLimit: usize) -> Vec<(usize, usize)> {
    let mut board = board_.clone();
    let (mut matrix_a, mut matrix_x, mut matrix_b) = refresh_matrix(&board);
    let ans = solve_direct(&mut matrix_a, &mut matrix_x, &mut matrix_b, &mut board);
    let mut not_mine = vec![];
    for i in ans.0 {
        board[i.0][i.1] = 12;
        not_mine.push(i);
    }
    let (matrix_a, matrix_x, matrix_b) = refresh_matrix(&board);
    let ans = solve_minus(&matrix_a, &matrix_x, &matrix_b, &mut board);
    for i in ans.0 {
        board[i.0][i.1] = 12;
        not_mine.push(i);
    }
    let (matrix_as, matrix_xs, matrix_bs, _, _) = refresh_matrixs(&board);
    let ans = solve_enumerate(&matrix_as, &matrix_xs, &matrix_bs, &mut board, enuLimit);
    for i in ans.0 {
        board[i.0][i.1] = 12;
        not_mine.push(i);
    }
    not_mine
}

// pub fn solve_all_notmine(matrix_a: Vec<Vec<i32>>, matrix_x: Vec<(usize, usize)>, matrix_b: Vec<i32>, enuLimit: usize) ->Vec<(usize, usize)> {
// 依据方程求出所有是雷和非雷的位置
//     let ans = SolveDirect(&mut matrix_a, &mut matrix_x, &mut matrix_b, &mut board);
//     let mut not_mine = vec![];
//     for i in ans.0 {
//         board[i.0][i.1] = 12;
//         not_mine.push(i);
//     }
//     let (matrix_a, matrix_x, matrix_b) = refresh_matrix(&board);
//     let ans = SolveMinus(&matrix_a, &matrix_x, &matrix_b, &mut board);
//     for i in ans.0 {
//         board[i.0][i.1] = 12;
//         not_mine.push(i);
//     }
//     let (matrix_as, matrix_xs, matrix_bs, _, _) = refresh_matrixs(&board);
//     let ans = SolveEnumerate(&matrix_as, &matrix_xs, &matrix_bs, &mut board, enuLimit);
//     for i in ans.0 {
//         board[i.0][i.1] = 12;
//         not_mine.push(i);
//     }
//     not_mine
// }
