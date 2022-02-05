use crate::utils::{
    cal3BV, cal3BV_exp, cal_table_minenum_enum, cal_table_minenum_recursion, chunk_matrixes,
    combine, enuOneStep, enum_comb, laymine_number, laymine_op_number, legalize_board,
    refresh_board, refresh_matrix, refresh_matrixs, refresh_matrixses, unsolvable_structure,
    BigNumber, C_query, C,
};

#[cfg(feature = "js")]
use crate::utils::js_shuffle;

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

/// 双集合判雷引擎。
/// - 输入：3个矩阵、局面。
/// - 返回：是雷、非雷的格子，在传入的局面上标是雷（11）和非雷（12）。  
/// - 注意：会维护系数矩阵、格子矩阵和数字矩阵，删、改、分段。
pub fn solve_minus(
    As: &mut Vec<Vec<Vec<i32>>>,
    xs: &mut Vec<Vec<(usize, usize)>>,
    bs: &mut Vec<Vec<i32>>,
    board_of_game: &mut Vec<Vec<i32>>,
) -> (Vec<(usize, usize)>, Vec<(usize, usize)>) {
    let block_num = bs.len();
    // let mut flag = false;
    let mut not_mine = vec![];
    let mut is_mine = vec![];
    let mut remove_blocks_id = vec![];
    for b in (0..block_num).rev() {
        let mut not_mine_rel = vec![];
        let mut is_mine_rel = vec![];
        let mut matrixColumn = xs[b].len();
        let mut matrixRow = bs[b].len();
        if matrixRow <= 1 {
            continue; // 整段只有一个数字，比如角落的1
        }
        for i in 1..matrixRow {
            for j in 0..i {
                let mut ADval1 = vec![];
                let mut ADvaln1 = vec![];
                let mut FlagAdj = false;
                for k in 0..matrixColumn {
                    if As[b][i][k] >= 1 && As[b][j][k] >= 1 {
                        FlagAdj = true;
                        continue;
                    }
                    if As[b][i][k] - As[b][j][k] == 1 {
                        ADval1.push(k)
                    } else if As[b][i][k] - As[b][j][k] == -1 {
                        ADvaln1.push(k)
                    }
                }
                if FlagAdj {
                    let bDval = bs[b][i] - bs[b][j];
                    if ADval1.len() as i32 == bDval {
                        is_mine_rel.append(&mut ADval1);
                        not_mine_rel.append(&mut ADvaln1);
                    } else if ADvaln1.len() as i32 == -bDval {
                        is_mine_rel.append(&mut ADvaln1);
                        not_mine_rel.append(&mut ADval1);
                    }
                }
            }
        }
        is_mine_rel.sort();
        is_mine_rel.dedup();
        not_mine_rel.sort();
        not_mine_rel.dedup();
        for i in &not_mine_rel {
            not_mine.push(xs[b][*i]);
            board_of_game[xs[b][*i].0][xs[b][*i].1] = 12;
        }
        for i in &is_mine_rel {
            is_mine.push(xs[b][*i]);
            board_of_game[xs[b][*i].0][xs[b][*i].1] = 11;
            for j in 0..As[b].len() {
                bs[b][j] -= As[b][j][*i];
            }
        }
        let mut del_id = not_mine_rel;
        del_id.append(&mut is_mine_rel);
        del_id.sort_by(|a, b| b.cmp(a));
        del_id.dedup();
        for i in del_id {
            xs[b].remove(i);
            for jj in 0..As[b].len() {
                As[b][jj].remove(i);
            }
        }
        if xs[b].is_empty() {
            remove_blocks_id.push(b);
        }
    }

    for b in remove_blocks_id {
        As.remove(b);
        bs.remove(b);
        xs.remove(b);
    }
    let (mut not, mut is) = solve_direct(As, xs, bs, board_of_game); // 没错，双集合判雷的最后一步是用单集合再过一轮。理由：（1）这样才不会报错（2）单集合复杂度很低，不费事
    not_mine.append(&mut not);
    is_mine.append(&mut is);
    chunk_matrixes(As, xs, bs);
    (not_mine, is_mine)
}

/// 单集合判雷引擎。
/// - 输入：3个矩阵、局面。
/// - 返回：是雷、非雷的格子，在传入的局面上标是雷（11）和非雷（12）。  
/// - 注意：会维护系数矩阵、格子矩阵和数字矩阵，删、改、分段。
pub fn solve_direct(
    As: &mut Vec<Vec<Vec<i32>>>,
    xs: &mut Vec<Vec<(usize, usize)>>,
    bs: &mut Vec<Vec<i32>>,
    board_of_game: &mut Vec<Vec<i32>>,
) -> (Vec<(usize, usize)>, Vec<(usize, usize)>) {
    let mut is_mine = vec![];
    let mut not_mine = vec![];

    let block_num = bs.len();
    for b in (0..block_num).rev() {
        let mut matrixColumn = xs[b].len();
        let mut matrixRow = bs[b].len();
        for i in (0..matrixRow).rev() {
            if As[b][i].iter().sum::<i32>() == bs[b][i] {
                for k in (0..matrixColumn).rev() {
                    if As[b][i][k] >= 1 {
                        is_mine.push((xs[b][k].0, xs[b][k].1));
                        board_of_game[xs[b][k].0][xs[b][k].1] = 11;
                        xs[b].remove(k);
                        for t in 0..matrixRow {
                            bs[b][t] -= As[b][t][k];
                            As[b][t].remove(k);
                        }
                        matrixColumn -= 1;
                    }
                }
                As[b].remove(i);
                bs[b].remove(i);
                matrixRow -= 1;
            }
        }
        for i in (0..matrixRow).rev() {
            if bs[b][i] == 0 {
                for k in (0..matrixColumn).rev() {
                    if As[b][i][k] >= 1 {
                        not_mine.push(xs[b][k]);
                        board_of_game[xs[b][k].0][xs[b][k].1] = 12;
                        xs[b].remove(k);
                        for t in 0..matrixRow {
                            As[b][t].remove(k);
                        }
                        matrixColumn -= 1;
                    }
                }
                As[b].remove(i);
                bs[b].remove(i);
                matrixRow -= 1;
            }
        }
        if bs[b].is_empty() {
            As.remove(b);
            bs.remove(b);
            xs.remove(b);
        }
    }
    chunk_matrixes(As, xs, bs);
    (not_mine, is_mine)
}

/// 游戏局面概率计算引擎。  
/// - 输入：局面、未被标出的雷数。未被标出的雷数大于等于1时，理解成实际数量；小于1时理解为比例。  
/// - 注意：局面中可以标雷（11）和非类（12），但必须全部标对。  
/// - 注意：若超出枚举长度，返回空向量。  
/// - 返回：所有边缘格子是雷的概率、内部未知格子是雷的概率、局面中总未知雷数（未知雷数 = 总雷数 - 已经标出的雷）的范围。  
/// - 注意：若没有内部未知区域，返回NaN。
pub fn cal_possibility(
    board_of_game: &Vec<Vec<i32>>,
    mut mine_num: f64,
) -> Result<(Vec<((usize, usize), f64)>, f64, [usize; 3]), usize> {
    let mut p = vec![];
    let mut table_cell_minenum_s: Vec<Vec<Vec<usize>>> = vec![];
    // 每段每格雷数表：记录了每段每格（或者地位等同的复合格）、每种总雷数下的是雷情况数
    let mut comb_relp_s = vec![]; // 记录了方格的组合关系
                                  // let mut enum_comb_table_s = vec![];
    let mut table_minenum_s: Vec<[Vec<usize>; 2]> = vec![];
    // 每段雷数分布表：记录了每段（不包括内部段）每种总雷数下的是雷总情况数
    // 例如：[[[17, 18, 19, 20, 21, 22, 23, 24], [48, 2144, 16872, 49568, 68975, 48960, 16608, 2046]]]
    let (matrix_a_s, matrix_x_s, matrix_b_s, unknow_block, is_mine_num) =
        refresh_matrixs(&board_of_game);
    let block_num = matrix_a_s.len(); // 整个局面被分成的段数

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
    // 分段枚举后，根据雷数限制，删除某些情况
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
    } // 第一步，整理出每段每格雷数情况表、每段雷数分布表、每段雷分布情况总数表
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
    // 第二步，整理内部未知段雷数分布表，并筛选。这样内部未知雷段和边缘雷段的地位视为几乎等同，但数据结构不同
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
    // 第三步，枚举每段雷数情况索引表：行代表每种情况，列代表每段雷数的索引，最后一列代表未知区域雷数
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
    // 第四步，计算每段其他雷数情况表
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

/// 计算局面中各位置是雷的概率，按照所在的位置返回。
/// # Example
/// - 用rust调用时的示例：
/// ```rust
/// let mut game_board = vec![
///     vec![10, 10, 1, 1, 10, 1, 0, 0],
///     vec![10, 10, 1, 10, 10, 3, 2, 1],
///     vec![10, 10, 10, 10, 10, 10, 10, 10],
///     vec![10, 10, 10, 10, 10, 10, 10, 10],
///     vec![10, 10, 10, 10, 10, 10, 10, 10],
///     vec![10, 10, 10, 10, 2, 10, 10, 10],
///     vec![10, 10, 10, 10, 10, 10, 10, 10],
///     vec![10, 10, 10, 10, 10, 10, 10, 10],
/// ];
/// let ans = cal_possibility(&game_board, 10.0);
/// print!("设置雷数为10，概率计算引擎的结果为：{:?}", ans);
/// let ans = cal_possibility(&game_board, 0.15625);
/// print!("设置雷的比例为15.625%，概率计算引擎的结果为：{:?}", ans);
/// // 对局面预标记，以加速计算
/// mark_board(&mut game_board);
/// let ans = cal_possibility_onboard(&game_board, 10.0);
/// print!("设置雷的比例为10，与局面位置对应的概率结果为：{:?}", ans);
/// ```
/// - 用Python调用时的示例：
/// ```python
/// import ms_toollib as ms
///
/// game_board = [
///     [10, 10, 1, 1, 10, 1, 0, 0],
///     [10, 10, 1, 10, 10, 3, 2, 1],
///     [10, 10, 10, 10, 10, 10, 10, 10],
///     [10, 10, 10, 10, 10, 10, 10, 10],
///     [10, 10, 10, 10, 10, 10, 10, 10],
///     [10, 10, 10, 10, 2, 10, 10, 10],
///     [10, 10, 10, 10, 10, 10, 10, 10],
///     [10, 10, 10, 10, 10, 10, 10, 10],
///     ];
/// ans = ms.cal_possibility(game_board, 10);
/// print("设置雷数为10，概率计算引擎的结果为：", ans);
/// ans = ms.cal_possibility(game_board, 0.15625);
/// print("设置雷的比例为15.625%，概率计算引擎的结果为：", ans);
/// # 对局面预标记，以加速计算
/// ms.mark_board(game_board);
/// ans = ms.cal_possibility_onboard(game_board, 10.0);
/// print("设置雷的比例为10，与局面位置对应的概率结果为：", ans);
/// ```
pub fn cal_possibility_onboard(
    board_of_game: &Vec<Vec<i32>>,
    mine_num: f64,
) -> Result<(Vec<Vec<f64>>, [usize; 3]), usize> {
    let mut p = vec![vec![-1.0; board_of_game[0].len()]; board_of_game.len()];
    let pp = cal_possibility(&board_of_game, mine_num)?;
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
            } else if p[r][c] < -0.5 {
                p[r][c] = 0.0;
            }
        }
    }
    Ok((p, pp.2))
}

/// 计算开空概率算法。  
/// - 输入：局面、未被标出的雷数、坐标（可以同时输入多个）。  
/// - 返回：坐标处开空的概率。  
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

/// 埋雷，使得起手位置必为空。  
/// - 输入：行数、列数、雷数、起手位置行数、起手位置列数、最小3BV、最大3BV、最大尝试次数、模式（保留）。  
/// - 输出：局面和参数，分别为：是否满足3BV上下限、3BV、尝试次数。  
/// # Example
/// ```
/// laymine_op(16, 30, 99, 0, 0, 100, 381, 10000, 0)
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

/// 枚举法判雷引擎。  
/// - 输入：分段好的矩阵、局面、枚举长度限制。  
/// - 输出：是雷、不是雷的位置。  
/// # Example
/// ```rust
/// let mut game_board = vec![
///     vec![0, 0, 1, 10, 10, 10, 10, 10],
///     vec![0, 0, 2, 10, 10, 10, 10, 10],
///     vec![1, 1, 3, 11, 10, 10, 10, 10],
///     vec![10, 10, 4, 10, 10, 10, 10, 10],
///     vec![10, 10, 10, 10, 10, 10, 10, 10],
///     vec![10, 10, 10, 10, 10, 10, 10, 10],
///     vec![10, 10, 10, 10, 10, 10, 10, 10],
///     vec![10, 10, 10, 10, 10, 10, 10, 10],
/// ];
/// let (As, xs, bs, _, _) = refresh_matrixs(&game_board);
/// let ans = solve_enumerate(&As, &xs, &bs, 30);
/// print!("{:?}", ans);
/// ```
/// - 注意：不修改输入进来的局面，即不帮助标雷（这个设计后续可能修改）；也不维护3个矩阵。因为枚举引擎是最后使用的  
/// - 注意：超出枚举长度限制是未定义的行为，算法不一定会得到足够多的结果  
pub fn solve_enumerate(
    As: &Vec<Vec<Vec<i32>>>,
    xs: &Vec<Vec<(usize, usize)>>,
    bs: &Vec<Vec<i32>>,
) -> (Vec<(usize, usize)>, Vec<(usize, usize)>) {
    if bs.is_empty() {
        return (vec![], vec![]);
    }
    let mut not_mine = vec![];
    let mut is_mine = vec![];
    let block_num = xs.len();

    let mut comb_relp_s = vec![];
    let mut matrixA_squeeze_s: Vec<Vec<Vec<i32>>> = vec![];
    let mut matrixx_squeeze_s: Vec<Vec<(usize, usize)>> = vec![];
    for i in 0..block_num {
        if xs[i].len() > ENUM_LIMIT {
            return (not_mine, is_mine);
        }
        let (matrixA_squeeze, matrixx_squeeze, combination_relationship) = combine(&As[i], &xs[i]);
        comb_relp_s.push(combination_relationship);
        matrixA_squeeze_s.push(matrixA_squeeze);
        matrixx_squeeze_s.push(matrixx_squeeze);
    }
    // println!("As{:?}", As);
    // println!("matrixA_squeeze_s{:?}", matrixA_squeeze_s);
    // println!("matrixx_squeeze_s{:?}", matrixx_squeeze_s);
    // println!("bs{:?}", bs);

    use crate::ENUM_LIMIT;
    for i in 0..block_num {
        let (table_minenum_i, table_cell_minenum_i) = cal_table_minenum_recursion(
            &matrixA_squeeze_s[i],
            &matrixx_squeeze_s[i],
            &bs[i],
            &comb_relp_s[i],
        )
        .unwrap();
        for jj in 0..table_cell_minenum_i[0].len() {
            let mut s_num = 0; // 该合成格子的总情况数
            for ii in 0..table_cell_minenum_i.len() {
                s_num += table_cell_minenum_i[ii][jj];
            }
            if s_num == 0 {
                for kk in &comb_relp_s[i][jj] {
                    not_mine.push(xs[i][*kk]);
                }
            } else if s_num == table_minenum_i[1].iter().sum::<usize>() * comb_relp_s[i][jj].len() {
                for kk in &comb_relp_s[i][jj] {
                    is_mine.push(xs[i][*kk]);
                }
            }
        }
    }
    (not_mine, is_mine)
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

/// <span id="is_solvable">从指定位置开始扫，判断局面是否无猜。  
/// - 注意：周围一圈都是雷，那么若中间是雷不算猜，若中间不是雷算有猜。  
/// - 注意：不考虑剩余雷数。
pub fn is_solvable(Board: &Vec<Vec<i32>>, x0: usize, y0: usize) -> bool {
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
    refresh_board(&Board, &mut BoardofGame, vec![(x0, y0)]);
    if isVictory(&BoardofGame, &Board) {
        return true; // 暂且认为点一下就扫开也是可以的
    }
    let mut not_mine;
    loop {
        let (mut As, mut xs, mut bs, _, _) = refresh_matrixs(&BoardofGame);
        let ans = solve_direct(&mut As, &mut xs, &mut bs, &mut BoardofGame);
        not_mine = ans.0;
        if !(not_mine.is_empty() && ans.1.is_empty()) {
            let ans = solve_minus(&mut As, &mut xs, &mut bs, &mut BoardofGame);
            not_mine = ans.0;
            if !(not_mine.is_empty() && ans.1.is_empty()) {
                let ans = solve_enumerate(&As, &xs, &bs);
                if !(ans.0.is_empty() && ans.1.is_empty()) {
                    return false;
                }
            }
        }
        refresh_board(&Board, &mut BoardofGame, not_mine);
        if isVictory(&BoardofGame, &Board) {
            return true;
        }
    }
}

/// 删选法多（8）线程无猜埋雷。  
/// - 输入：3BV下限、上限，最大尝试次数。  
/// - 返回: (是否成功、3BV、该线程的尝试次数)。  
/// - 注意：若不成功返回最后生成的局面，此时则不一定无猜。
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
) -> (Vec<Vec<i32>>, [usize; 3]) {
    let mut parameters = [0, 0, 0];
    let mut game_board = vec![vec![0; column]; row];
    let mut handles = vec![];
    let flag_exit = Arc::new(Mutex::new(0));
    let (tx, rx) = mpsc::channel(); // mpsc 是多个发送者，一个接收者
    for ii in (1..9).rev() {
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
                } // 这段用花括号控制生命周期
                let Board_ = laymine_op_number(row, column, MineNum, x0, y0);
                counter += 1;
                if is_solvable(&Board_, x0, y0) {
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

/// 删选法单线程无猜埋雷。  
/// - 输入：高、宽、雷数、起手行数、起手列数、3BV下限、上限、最大尝试次数、最大枚举长度。  
/// - 返回：是否成功。  
/// - 注意：若不成功返回最后生成的局面，此时则不一定无猜。
pub fn laymine_solvable(
    row: usize,
    column: usize,
    MineNum: usize,
    x0: usize,
    y0: usize,
    Min3BV: usize,
    Max3BV: usize,
    MaxTimes: usize,
) -> (Vec<Vec<i32>>, Vec<usize>) {
    let mut times = 0;
    let mut Parameters = vec![];
    let mut Board;
    let mut Num3BV;
    while times < MaxTimes {
        Board = laymine_op_number(row, column, MineNum, x0, y0);
        times += 1;
        if is_solvable(&Board, x0, y0) {
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
/// 返回局面、是否成功
pub fn laymine_solvable_adjust(
    row: usize,
    column: usize,
    mine_num: usize,
    x0: usize,
    y0: usize,
) -> (Vec<Vec<i32>>, bool) {
    // 利用局面调整算法，无猜埋雷
    let mut board = vec![vec![0; column]; row];
    let mut board_of_game = vec![vec![10; column]; row];
    board_of_game[x0][y0] = 0;
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
        let t = laymine_number(row, column, mine_num, x0, y0);
        return (t, false);
    }
    'o: for time in 0..10 {
        let remain_mine_num = mine_num;
        let remain_not_mine_num = row * column - area_op;
        let mut cells_plan_to_click = vec![];
        for j in max(1, x0) - 1..min(row, x0 + 2) {
            for k in max(1, y0) - 1..min(column, y0 + 2) {
                if j != x0 && k != y0 {
                    cells_plan_to_click.push((j, k));
                    board[j][k] = 22;
                }
            }
        }

        let mut area_current_adjust = get_adjust_area(&board);
        loop {
            // 每一次代表点开一批格子
            let mut mine_num_reduce = 0;
            let mut mine_num_reduce_plus_time_left = 3;

            loop {
                // 每一次循环代表对局部一次调整
                let mine_num = (area_current_adjust.len() as f64 * remain_mine_num as f64
                    / remain_not_mine_num as f64) as usize
                    - mine_num_reduce;
                if mine_num < 0 || mine_num > remain_mine_num {
                    continue 'o;
                }
                mine_num_reduce_plus_time_left -= 1;
                if mine_num_reduce_plus_time_left <= 0 {
                    mine_num_reduce_plus_time_left = 3;
                    mine_num_reduce += 1;
                }
                adjust_the_area_on_board(
                    &mut board,
                    &area_current_adjust,
                    &cells_plan_to_click,
                    mine_num,
                );
                let mut board_of_game_clone = board_of_game.clone();
                for &(x, y) in &cells_plan_to_click {
                    board_of_game_clone[x][y] = board[x][y];
                }
            }
        }
    }
    (vec![], false)
}

// 在指定的局部（area_current_adjust）埋雷，并刷新board上（cells_plan_to_click）
// 处的数字
fn adjust_the_area_on_board(
    board: &mut Vec<Vec<i32>>,
    area_current_adjust: &Vec<(usize, usize)>,
    cells_plan_to_click: &Vec<(usize, usize)>,
    mine_num: usize,
) {
    let row = board.len();
    let column = board[0].len();
    let mut b = vec![0; area_current_adjust.len() - mine_num];
    b.append(&mut vec![-1; mine_num]);

    #[cfg(any(feature = "py", feature = "rs"))]
    let mut rng = thread_rng();
    #[cfg(any(feature = "py", feature = "rs"))]
    b.shuffle(&mut rng);

    #[cfg(feature = "js")]
    b.shuffle_();

    for id in 0..area_current_adjust.len() {
        let x = area_current_adjust[id].0;
        let y = area_current_adjust[id].1;
        board[x][y] = b[id];
    }
    for (x, y) in cells_plan_to_click {
        let mut number = 0;
        for m in max(1, *x) - 1..min(row, *x + 2) {
            for n in max(1, *y) - 1..min(column, *y + 2) {
                if board[m][n] == -1 {
                    number += 1;
                }
            }
        }
        board[*x][*y] = number;
    }
}

fn get_adjust_area(board_of_game: &Vec<Vec<i32>>) -> Vec<(usize, usize)> {
    let row = board_of_game.len();
    let column = board_of_game[0].len();
    let mut a = vec![];
    for x in 0..row {
        for y in 0..column {
            if board_of_game[x][y] == 10 {
                'o: for m in max(1, x) - 1..min(row, x + 2) {
                    for n in max(1, y) - 1..min(column, y + 2) {
                        if board_of_game[m][n] == 22 {
                            a.push((x, y));
                            break 'o;
                        }
                    }
                }
            }
        }
    }
    a
}

/// 适用于游戏的标准埋雷。  
/// - 输入：依次为行、列、雷数、起手位置的第几行、第几列  
/// - 共识：标准埋雷规则为起手不一定开空，但必不为雷  
/// - 返回：二维可变长的向量，用0~8代表数字，-1代表雷  
/// method = 0筛选算法；1调整算法（目前无效并保留）  
/// # Example
/// - 用rust调用时的示例：
/// ```rust
/// laymine(16, 30, 99, 0, 0, 100, 381, 1000, "");
/// ```
/// - 用python调用时的示例：
/// ```python
/// import ms_toollib as ms
/// ms.laymine(16, 30, 99, 0, 0, 100, 381, 1000, '');
/// ```
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

// 对高级3BV做采样。16线程。
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

/// <span id="OBR_board">光学局面识别引擎。  
/// - 输入：依次为列向量形式的三通道的像素数据、图像的高度、宽度。  
/// - 以下提供一段用python调用时的示例：  
/// ```python
/// # pip install ms_toollib --upgrade
/// import ms_toollib
/// import matplotlib.image as mpimg
/// import numpy as np
/// file = r'C:\Users\jia32\Desktop\无标题.png'# 彩色图片
/// lena = mpimg.imread(file)
/// (height, width) = lena.shape[:2]
/// lena = np.concatenate((lena, 255.0 * np.ones((height, width, 1))), 2)
/// lena = (np.reshape(lena, -1) * 255).astype(np.uint32)
/// board = ms_toollib.OBR_board(lena, height, width)
/// print(np.array(board))# 打印识别出的局面
/// poss = ms_toollib.cal_possibility(board, 99)
/// print(poss)# 用雷的总数计算概率
/// poss_onboard = ms_toollib.cal_possibility_onboard(board, 99)
/// print(poss_onboard)# 用雷的总数计算概率，输出局面对应的位置
/// poss_ = ms_toollib.cal_possibility_onboard(board, 0.20625)
/// print(poss_)# 用雷的密度计算概率
/// ```
/// 细节：对于标雷、游戏结束后的红雷、叉雷，一律识别成10，即没有打开。
/// 注意：必须配合“params.onnx”参数文件调用。  
/// 注意：由于利用了神经网络技术，可能发生识别错误，此时输出是不一定合法的局面。
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

/// 对局面用单集合、双集合判雷引擎，快速标雷、标非雷，以供概率计算引擎处理。  
/// 相当于一种预处理，即先标出容易计算的。  
/// - 注意：在rust中，cal_possibility往往需要和mark_board搭配使用，而在其他语言（python）中可能不需要如此！这是由于其ffi不支持原地操作。
pub fn mark_board(board_of_game: &mut Vec<Vec<i32>>) {
    let (mut As, mut xs, mut bs, _, _) = refresh_matrixs(&board_of_game);
    solve_direct(&mut As, &mut xs, &mut bs, board_of_game);
    solve_minus(&mut As, &mut xs, &mut bs, board_of_game);
}

/// 求出游戏局面中所有非雷、是雷的位置。  
/// - 注意：局面中可以有标雷，但不能有错误！
pub fn get_all_not_and_is_mine_on_board(
    As: &mut Vec<Vec<Vec<i32>>>,
    xs: &mut Vec<Vec<(usize, usize)>>,
    bs: &mut Vec<Vec<i32>>,
    board_of_game: &mut Vec<Vec<i32>>,
) -> (Vec<(usize, usize)>, Vec<(usize, usize)>) {
    let mut ans = solve_direct(As, xs, bs, board_of_game);
    let mut is_mine = vec![];
    let mut not_mine = vec![];
    not_mine.append(&mut ans.0);
    is_mine.append(&mut ans.1);
    let mut ans = solve_minus(As, xs, bs, board_of_game);
    not_mine.append(&mut ans.0);
    is_mine.append(&mut ans.1);
    let mut ans = solve_enumerate(As, xs, bs);
    not_mine.append(&mut ans.0);
    is_mine.append(&mut ans.1);
    (not_mine, is_mine)
}

// // 求出该块所有的解。
// // 一块(block)包含多段(segment)
// fn get_all_not_mine_on_block(As: &Vec<Vec<Vec<i32>>>, xs: &Vec<Vec<(usize, usize)>>, bs: &Vec<Vec<i32>>) -> bool {
//     let (n, i) = solve_direct(As, xs, bs);
//     if n.is_empty() {
//         let (n, i) = solve_minus(As, xs, bs);
//         if n.is_empty() {
//             let (n, i) = solve_enumerate(As, xs, bs);
//             if n.is_empty() {
//                 return false
//             }
//         }
//     }
//     true
// }

/// 判断是否为可能可以（区别于必然可以）判雷时的猜雷；对应弱无猜规则。  
/// - 前提：点在未知格上，即10。  
/// 情况表：1 -> 正确的判雷。  
/// 2 -> 必要的猜雷。  
/// 3 -> 不必要的猜雷。  
/// 4 -> 踩到必然的雷。
pub fn is_guess_while_needless(board_of_game: &mut Vec<Vec<i32>>, xy: &(usize, usize)) -> i32 {
    let mut flag_need = true;
    let (mut Ases, mut xses, mut bses) = refresh_matrixses(&board_of_game);
    let t = xses
        .iter()
        .position(|r| r.iter().any(|x| x.contains(&xy)))
        .unwrap();
    let mut As = &mut Ases[t];
    let mut xs = &mut xses[t];
    let mut bs = &mut bses[t];
    let (n, _) = solve_direct(As, xs, bs, board_of_game);
    flag_need = n.is_empty();
    match board_of_game[xy.0][xy.1] {
        12 => return 1,
        11 => return 4,
        _ => {
            let (n, _) = solve_minus(As, xs, bs, board_of_game);
            flag_need = flag_need || n.is_empty();
            match board_of_game[xy.0][xy.1] {
                12 => return 1,
                11 => return 4,
                _ => {
                    let (n, i) = solve_enumerate(As, xs, bs);
                    flag_need = flag_need || n.is_empty();
                    if n.contains(xy) {
                        return 1;
                    } else if i.contains(xy) {
                        return 4;
                    } else if flag_need {
                        return 2;
                    } else {
                        return 3;
                    }
                }
            }
        }
    }
}

/// 判断是否为判雷；对应强无猜规则。
/// - 前提：对打开非雷或标记是雷的行为判断。   
/// - 不仅可以判断是雷，也可以判断非雷。
pub fn is_able_to_solve(board_of_game: &mut Vec<Vec<i32>>, xy: &(usize, usize)) -> bool {
    let (mut As, mut xs, mut bs, _, _) = refresh_matrixs(&board_of_game);
    solve_direct(&mut As, &mut xs, &mut bs, board_of_game);
    if board_of_game[xy.0][xy.1] == 11 || board_of_game[xy.0][xy.1] == 12 {
        return true;
    }
    solve_minus(&mut As, &mut xs, &mut bs, board_of_game);
    if board_of_game[xy.0][xy.1] == 11 || board_of_game[xy.0][xy.1] == 12 {
        return true;
    }
    let (n, i) = solve_enumerate(&As, &xs, &bs);
    if i.contains(xy) || n.contains(xy) {
        return true;
    }
    false
}

// pub fn each_block_is_able_to_judge(board_of_game: &mut Vec<Vec<i32>>, xy: &(usize, usize)) -> Vec<bool> {
//     vec![true]
// let (mut As, mut xs, mut bs, _, _) = refresh_matrixs(&board_of_game);
// solve_direct(&mut As, &mut xs, &mut bs, board_of_game);
// if board_of_game[xy.0][xy.1] == 11 || board_of_game[xy.0][xy.1] == 12 {
//     return true;
// }
// solve_minus(&mut As, &mut xs, &mut bs, board_of_game);
// if board_of_game[xy.0][xy.1] == 11 || board_of_game[xy.0][xy.1] == 12 {
//     return true;
// }
// let (n, i) = solve_enumerate(&As, &xs, &bs);
// if i.contains(xy) || n.contains(xy) {
//     return true;
// }
// false
// }
