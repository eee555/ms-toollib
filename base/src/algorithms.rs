use crate::utils::{
    cal_bbbv_exp, cal_table_minenum_recursion, chunk_matrixes, combine, find_a_border_cell,
    laymine, laymine_op, legalize_board, refresh_board, refresh_matrixs, refresh_matrixses,
    unsolvable_structure, BigNumber, C,
};

use crate::videos::{MinesweeperBoard, GameBoardState};

#[cfg(feature = "js")]
use crate::utils::js_shuffle;

#[cfg(any(feature = "py", feature = "rs"))]
use crate::OBR::ImageBoard;

use itertools::Itertools;

#[cfg(any(feature = "py", feature = "rs"))]
use rand::seq::SliceRandom;
#[cfg(any(feature = "py", feature = "rs"))]
use rand::thread_rng;

#[cfg(any(feature = "py", feature = "rs"))]
use rand::prelude::*;

use std::cmp::{max, min};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

#[cfg(any(feature = "py", feature = "rs"))]
use tract_ndarray::Array;

#[cfg(any(feature = "py", feature = "rs"))]
use tract_onnx::prelude::*;

use crate::ENUM_LIMIT;

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
) -> Result<(Vec<(usize, usize)>, Vec<(usize, usize)>), usize> {
    let block_num = bs.len();
    // let mut flag = false;
    let mut not_mine = vec![];
    let mut is_mine = vec![];
    let mut remove_blocks_id = vec![];
    for b in (0..block_num).rev() {
        let mut not_mine_rel = vec![];
        let mut is_mine_rel = vec![];
        let matrix_column = xs[b].len();
        let matrix_row = bs[b].len();
        if matrix_row <= 1 {
            continue; // 整段只有一个数字，比如角落的1
        }
        for i in 1..matrix_row {
            for j in 0..i {
                let mut ADval1 = vec![];
                let mut ADvaln1 = vec![];
                let mut FlagAdj = false;
                for k in 0..matrix_column {
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
    let (mut not, mut is) = solve_direct(As, xs, bs, board_of_game)?; // 没错，双集合判雷的最后一步是用单集合再过一轮。理由：（1）这样才不会报错（2）单集合复杂度很低，不费事
    not_mine.append(&mut not);
    is_mine.append(&mut is);
    chunk_matrixes(As, xs, bs);
    Ok((not_mine, is_mine))
}

/// 单集合判雷引擎。
/// - 输入：3个矩阵、局面。
/// - 返回：非雷、是雷的格子，在传入的局面上标是雷（11）和非雷（12）。  
/// - 返回Err(6)表示：比如数字2的周围只有1个格子没打开  
/// - 注意：会维护系数矩阵、格子矩阵和数字矩阵，删、改、分段。
pub fn solve_direct(
    As: &mut Vec<Vec<Vec<i32>>>,
    xs: &mut Vec<Vec<(usize, usize)>>,
    bs: &mut Vec<Vec<i32>>,
    board_of_game: &mut Vec<Vec<i32>>,
) -> Result<(Vec<(usize, usize)>, Vec<(usize, usize)>), usize> {
    let mut is_mine = vec![];
    let mut not_mine = vec![];

    let block_num = bs.len();
    for b in (0..block_num).rev() {
        let mut matrix_column = xs[b].len();
        let mut matrix_row = bs[b].len();
        for i in (0..matrix_row).rev() {
            if As[b][i].iter().sum::<i32>() == bs[b][i] {
                for k in (0..matrix_column).rev() {
                    if As[b][i][k] >= 1 {
                        is_mine.push((xs[b][k].0, xs[b][k].1));
                        board_of_game[xs[b][k].0][xs[b][k].1] = 11;
                        xs[b].remove(k);
                        for t in 0..matrix_row {
                            bs[b][t] -= As[b][t][k];
                            As[b][t].remove(k);
                        }
                        matrix_column -= 1;
                    }
                }
                As[b].remove(i);
                bs[b].remove(i);
                matrix_row -= 1;
            }
        }
        for i in (0..matrix_row).rev() {
            if bs[b][i] == 0 {
                for k in (0..matrix_column).rev() {
                    if As[b][i][k] >= 1 {
                        not_mine.push(xs[b][k]);
                        board_of_game[xs[b][k].0][xs[b][k].1] = 12;
                        xs[b].remove(k);
                        for t in 0..matrix_row {
                            As[b][t].remove(k);
                        }
                        matrix_column -= 1;
                    }
                }
                As[b].remove(i);
                bs[b].remove(i);
                matrix_row -= 1;
            }
        }
        if bs[b].is_empty() {
            As.remove(b);
            bs.remove(b);
            xs.remove(b);
        }
    }
    let ans = bs.iter().find(|&b| match b.iter().find(|&&x| x < 0) {
        Some(_) => return true,
        None => return false,
    });
    match ans {
        Some(_) => return Err(6),
        None => {}
    }
    chunk_matrixes(As, xs, bs);
    Ok((not_mine, is_mine))
}

/// 游戏局面概率计算引擎。  
/// - 输入：局面、未被标出的雷数。未被标出的雷数大于等于1时，理解成实际数量；小于1时理解为比例。  
/// - 注意：局面中可以标雷（11）和非类（12），但必须全部标对。  
/// - 注意：若超出枚举长度（固定值55），则该区块的部分返回平均概率，且返回所需的枚举长度。  
/// - 返回：所有边缘格子是雷的概率、内部未知格子是雷的概率、局面中总未知雷数（未知雷数 = 总雷数 - 已经标出的雷）的范围、上述“所需的枚举长度”。  
/// - 注意：若没有内部未知区域，“内部未知格子是雷的概率”返回NaN。
/// - 局限：不能将所有矛盾的局面都检查出来。例如空中间出现一个数字，这种错误的局面，不能检查出来。
pub fn cal_possibility(
    board_of_game: &Vec<Vec<i32>>,
    minenum: f64,
) -> Result<(Vec<((usize, usize), f64)>, f64, [usize; 3], usize), usize> {
    // 如果超出枚举长度限制，记录并返回这个长度，以此体现局面的求解难度。
    let mut exceed_len = 0;
    let mut p = vec![];
    let mut table_cell_minenum_s: Vec<Vec<Vec<usize>>> = vec![];
    // 每段每格雷数表：记录了每段每格（或者地位等同的复合格）、每种总雷数下的是雷情况数
    let mut comb_relp_s = vec![]; // 记录了方格的组合关系
                                  // let mut enum_comb_table_s = vec![];
    let mut table_minenum_s: Vec<[Vec<usize>; 2]> = vec![];
    // 每段雷数分布表：记录了每段（不包括内部段）每种总雷数下的是雷总情况数
    // 例如：[[[17, 18, 19, 20, 21, 22, 23, 24], [48, 2144, 16872, 49568, 68975, 48960, 16608, 2046]]]
    let (mut matrix_a_s, mut matrix_x_s, mut matrix_b_s, mut unknow_block, is_minenum) =
        refresh_matrixs(&board_of_game);
    for i in (0..matrix_a_s.len()).rev() {
        let matrix_x_s_len = matrix_x_s[i].len();
        if matrix_x_s_len > exceed_len {
            exceed_len = matrix_x_s_len;
        }
        if matrix_x_s_len > ENUM_LIMIT {
            matrix_a_s.remove(i);
            matrix_x_s.remove(i);
            matrix_b_s.remove(i);
            unknow_block += matrix_x_s_len;
        }
    }

    let block_num = matrix_a_s.len(); // 整个局面被分成的段数
                                      // let mut block_num_calable = 0;

    let mut matrixA_squeeze_s: Vec<Vec<Vec<i32>>> = vec![];
    let mut matrixx_squeeze_s: Vec<Vec<(usize, usize)>> = vec![];
    // let mut min_max_minenum = [0, 0];
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
                // block_num_calable += 1;
            }
            Err(e) => {
                // 1: 这是不明显的，通过枚举才能发现的不合法、矛盾的的局面
                // 5: 这是明显的、不合法、矛盾的的局面
                return Err(e);
            }
        };
        // min_max_minenum[0] += table_minenum_i[0][0];
        // min_max_minenum[1] += table_minenum_i[0][table_minenum_i[0].len() - 1];

        table_cell_minenum_s.push(table_cell_minenum_i);
        table_minenum_s.push(table_minenum_i);
    } // 第一步，整理出每段每格雷数情况表、每段雷数分布表、每段雷分布情况总数表
    let mut min_minenum = 0;
    let mut max_minenum = 0;

    // let block_num = block_num_calable;

    for i in 0..block_num {
        min_minenum += table_minenum_s[i][0].iter().min().unwrap();
        max_minenum += table_minenum_s[i][0].iter().max().unwrap();
    }
    let minenum = if minenum < 1.0 {
        let mn = ((board_of_game.len() * board_of_game[0].len()) as f64 * minenum) as usize;
        min(
            max(mn - is_minenum, min_minenum),
            max_minenum + unknow_block,
        )
    } else {
        let mm = (minenum as usize).overflowing_sub(is_minenum);
        match mm.1 {
            false => mm.0,
            // 标雷环节没有查出错误，而且标了很多雷，算概率环节会返回17
            true => return Err(17),
        }
    };

    max_minenum = min(max_minenum, minenum);
    if max_minenum < min_minenum {
        return Err(3); // 这种错误，例如一共10个雷，却出现了3个不相邻的5
    }
    // println!("{:?}, {:?}, {:?}", max_minenum, minenum, min_minenum);
    let unknow_minenum: Vec<usize> =
        (minenum - max_minenum..min(minenum - min_minenum, unknow_block) + 1).collect();
    // 这里的写法存在极小的风险，例如边缘格雷数分布是0，1，3，而我们直接认为了可能有2
    let mut unknow_mine_s_num = vec![];
    for i in &unknow_minenum {
        unknow_mine_s_num.push(C(unknow_block, *i));
    }
    // 第二步，整理内部未知段雷数分布表，并筛选。这样内部未知雷段和边缘雷段的地位视为几乎等同，但数据结构不同
    table_minenum_s.push([unknow_minenum.clone(), vec![]]);
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
        if total_num != minenum {
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
            let mut s_mn = minenum; // 未知区域中的雷数
            for j in 0..block_num {
                if i != j {
                    s_num.mul_usize(table_minenum_s[j][1][s[j]]);
                }
                s_mn -= table_minenum_s[j][0][s[j]];
            }
            let ps = unknow_minenum.iter().position(|x| *x == s_mn).unwrap();
            s_num.mul_big_number(&unknow_mine_s_num[ps]);
            table_minenum_other[i][s[i]].add_big_number(&s_num);
        }
        let mut s_num = BigNumber { a: 1.0, b: 0 };
        let mut s_mn = minenum; // 未知区域中的雷数
        for j in 0..block_num {
            s_num.mul_usize(table_minenum_s[j][1][s[j]]);
            s_mn -= table_minenum_s[j][0][s[j]];
        }
        let ps = unknow_minenum.iter().position(|x| *x == s_mn).unwrap();
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
    for i in 0..unknow_minenum.len() {
        let mut u = table_minenum_other[block_num][i].clone();
        u.mul_big_number(&unknow_mine_s_num[i]);
        u.mul_usize(unknow_minenum[i]);
        u_s.add_big_number(&u);
    }
    let p_unknow = u_s.div_big_num(&T) / unknow_block as f64;
    // 第七步，计算内部未知区域是雷的概率

    Ok((
        p,
        p_unknow,
        [
            min_minenum + is_minenum,
            minenum + is_minenum,
            max_minenum + is_minenum + unknow_block,
        ],
        exceed_len,
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
    minenum: f64,
) -> Result<(Vec<Vec<f64>>, [usize; 3]), usize> {
    let mut p = vec![vec![-1.0; board_of_game[0].len()]; board_of_game.len()];
    let pp = cal_possibility(&board_of_game, minenum)?;
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
/// use ms_toollib::cal_is_op_possibility_cells;
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
    minenum: f64,
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
                    match cal_possibility_onboard(&board_of_game_modified, minenum) {
                        Ok((ppp, _)) => p = ppp,
                        Err(_) => {
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
/// let ans = solve_enumerate(&As, &xs, &bs, 40);
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

// 判断当前是否获胜，单次
// 游戏局面中必须没有标错的雷
fn is_victory(game_board: &Vec<Vec<i32>>, board: &Vec<Vec<i32>>) -> bool {
    let row = game_board.len();
    let col = game_board[0].len();
    for i in 0..row {
        for j in 0..col {
            if game_board[i][j] == 10 && board[i][j] != -1 {
                return false;
            }
        }
    }
    return true;
}

// 判断当前是否获胜，持续跟踪，提高效率
pub struct IsVictory {
    row: usize,
    column: usize,
    pointer_x: usize,
    pointer_y: usize,
}

impl IsVictory {
    pub fn new(row: usize, column: usize) -> IsVictory {
        IsVictory {
            row,
            column,
            pointer_x: 0,
            pointer_y: 0,
        }
    }
    fn is_victory(&mut self, game_board: &Vec<Vec<i32>>, board: &Vec<Vec<i32>>) -> bool {
        for j in self.pointer_y..self.column {
            if game_board[self.pointer_x][j] < 10 {
                if game_board[self.pointer_x][j] != board[self.pointer_x][j] {
                    return false; // 安全性相关（发生作弊）
                }
            }
            if game_board[self.pointer_x][j] >= 10 && board[self.pointer_x][j] != -1 {
                self.pointer_y = j;
                return false;
            }
        }
        for i in self.pointer_x + 1..self.row {
            for j in 0..self.column {
                if game_board[i][j] < 10 {
                    if game_board[i][j] != board[i][j] {
                        return false; // 安全性相关（发生作弊）
                    }
                }
                if game_board[i][j] >= 10 && board[i][j] != -1 {
                    self.pointer_x = i;
                    self.pointer_y = j;
                    return false;
                }
            }
        }
        true
    }
}

/// <span id="is_solvable">从指定位置开始扫，判断局面是否无猜。  
/// - 注意：周围一圈都是雷，那么若中间是雷不算猜，若中间不是雷算有猜。  
/// - 注意：不考虑剩余雷数。
pub fn is_solvable(board: &Vec<Vec<i32>>, x0: usize, y0: usize) -> bool {
    if board[x0][y0] == -1 {
        // 踩雷肯定是非无猜
        return false;
    }
    if unsolvable_structure(&board) {
        //若包含不可判雷结构，则不是无猜
        return false;
    }
    let row = board.len();
    let column = board[0].len();
    let mut game_board = vec![vec![10; column]; row];
    // 10是未打开，11是标雷
    // 局面大小必须超过6*6
    refresh_board(board, &mut game_board, vec![(x0, y0)]);
    let mut judge = IsVictory::new(row, column);
    if judge.is_victory(&game_board, &board) {
        return true; // 暂且认为点一下就扫开也是可以的
    }
    loop {
        let (mut As, mut xs, mut bs, _, _) = refresh_matrixs(&game_board);
        let ans = solve_direct(&mut As, &mut xs, &mut bs, &mut game_board).unwrap();
        let not_mine;
        if ans.0.is_empty() && ans.1.is_empty() {
            let ans = solve_minus(&mut As, &mut xs, &mut bs, &mut game_board).unwrap();
            if ans.0.is_empty() && ans.1.is_empty() {
                let ans = solve_enumerate(&As, &xs, &bs);
                if ans.0.is_empty() && ans.1.is_empty() {
                    return false;
                } else {
                    not_mine = ans.0
                }
                if !ans.1.is_empty() {
                    for (o, p) in ans.1 {
                        game_board[o][p] = 11;
                    }
                }
            } else {
                not_mine = ans.0
            }
        } else {
            not_mine = ans.0
        }
        refresh_board(board, &mut game_board, not_mine);
        if judge.is_victory(&game_board, &board) {
            return true;
        }
    }
}

/// <span id="is_solvable">从指定位置开始扫。  
/// - 注意：周围一圈都是雷，那么若中间是雷不算猜，若中间不是雷算有猜。  
/// - 注意：不考虑剩余雷数。
/// - 返回：游戏局面的残局、解决的bbbv数
pub fn try_solve(board: &Vec<Vec<i32>>, x0: usize, y0: usize) -> (Vec<Vec<i32>>, usize) {
    let row = board.len();
    let column = board[0].len();
    let mut game_board = vec![vec![10; column]; row];
    if board[x0][y0] == -1 {
        // 踩雷
        game_board[x0][y0] = 15;
        return (game_board, 0);
    }
    let mut minesweeper_board = MinesweeperBoard::<Vec<Vec<i32>>>::new(board.clone());
    let _ = minesweeper_board.step("lc", (x0, y0));
    let _ = minesweeper_board.step("lr", (x0, y0));
    // let mut judge = IsVictory::new(row, column);
    if is_victory(&game_board, &board) {
        return (minesweeper_board.game_board, 1); // 暂且认为点一下就扫开也是可以的
    }
    loop {
        let (mut As, mut xs, mut bs, _, _) = refresh_matrixs(&minesweeper_board.game_board);
        let ans =
            solve_direct(&mut As, &mut xs, &mut bs, &mut minesweeper_board.game_board).unwrap();
        let not_mine;
        if ans.0.is_empty() && ans.1.is_empty() {
            let ans =
                solve_minus(&mut As, &mut xs, &mut bs, &mut minesweeper_board.game_board).unwrap();
            if ans.0.is_empty() && ans.1.is_empty() {
                let ans = solve_enumerate(&As, &xs, &bs);
                if ans.0.is_empty() && ans.1.is_empty() {
                    return (minesweeper_board.game_board, minesweeper_board.bbbv_solved);
                } else {
                    not_mine = ans.0
                }
                if !ans.1.is_empty() {
                    for (o, p) in ans.1 {
                        minesweeper_board.game_board[o][p] = 11;
                    }
                }
            } else {
                not_mine = ans.0
            }
        } else {
            not_mine = ans.0
        }
        let mut operation = vec![];
        for n in not_mine{
            operation.push(("lc", n));
            operation.push(("lr", n));
        }
        let _  = minesweeper_board.step_flow(operation);
        if minesweeper_board.game_board_state == GameBoardState::Win{
            return (minesweeper_board.game_board, minesweeper_board.bbbv_solved);
        }
    }
}

/// 删选法多（8）线程无猜埋雷。对于雷密度很高的局面，多线程比单线程更快。  
/// - 输入：高、宽、雷数、第几行、第几列、最大尝试次数。  
/// - 返回: (是否成功、3BV、该线程的尝试次数)。  
/// - 注意：若不成功返回最后生成的局面，此时则不一定无猜。
/// - 局限：雷的密度无法设得很大。以高级的局面为例，雷数最多设到大约130左右。
/// - 用python调用时的示例：
/// ```python
/// import ms_toollib as ms
/// board = laymine_solvable_thread(16, 30, 99, 3, 20, 100000) # 在第3行、第20列开局
/// ```
#[cfg(any(feature = "py", feature = "rs"))]
pub fn laymine_solvable_thread(
    row: usize,
    column: usize,
    minenum: usize,
    x0: usize,
    y0: usize,
    mut max_times: usize,
) -> (Vec<Vec<i32>>, bool) {
    let mut game_board = vec![vec![0; column]; row];
    let mut handles = vec![];
    let flag_exit = Arc::new(Mutex::new(0));
    let (tx, rx) = mpsc::channel(); // mpsc 是多个发送者，一个接收者
                                    // println!("{:?}", thread::available_parallelism().unwrap());
    for ii in (1..=8).rev() {
        let tx_ = mpsc::Sender::clone(&tx);
        let max_time = max_times / ii;
        max_times -= max_time;
        let flag_exit = Arc::clone(&flag_exit);
        let handle = thread::spawn(move || {
            // let Num3BV;
            let mut counter = 0;
            let mut Board = vec![vec![0; column]; row];
            // let mut para = [0, 0, 0];
            while counter < max_time {
                {
                    let f = flag_exit.lock().unwrap();
                    if *f == 1 {
                        break;
                    }
                } // 这段用花括号控制生命周期
                let Board_ = laymine_op(row, column, minenum, x0, y0);
                counter += 1;
                if is_solvable(&Board_, x0, y0) {
                    for i in 0..row {
                        for j in 0..column {
                            Board[i][j] = Board_[i][j];
                        }
                    }
                    let mut f = flag_exit.lock().unwrap();
                    *f = 1;
                    tx_.send((Board, true)).unwrap();
                    break;
                }
            }
            let Board_ = laymine_op(row, column, minenum, x0, y0);
            // Num3BV = cal_bbbv(&Board_);
            tx_.send((Board_, false)).unwrap();
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
    let received = rx.recv().unwrap(); // 尝试次数仅仅为单个线程的次数，并不准
    for i in 0..row {
        for j in 0..column {
            game_board[i][j] = received.0[i][j];
        }
    }
    (game_board, received.1)
}

/// 删选法单线程无猜埋雷。不可以生成任意雷密度的无猜局面。但雷满足均匀分布。  
/// - 输入：高、宽、雷数、起手行数、起手列数、最大尝试次数。  
/// - 返回：是否成功。  
/// - 注意：若不成功返回最后生成的局面，此时则不一定无猜。
/// - 用python调用时的示例：
/// ```python
/// import ms_toollib as ms
/// board = laymine_solvable(16, 30, 99, 3, 20, 100000) # 在第3行、第20列开局
/// ```
pub fn laymine_solvable(
    row: usize,
    column: usize,
    minenum: usize,
    x0: usize,
    y0: usize,
    max_times: usize,
) -> (Vec<Vec<i32>>, bool) {
    let mut times = 0;
    let mut board;
    while times < max_times {
        board = laymine_op(row, column, minenum, x0, y0);
        times += 1;
        if is_solvable(&board, x0, y0) {
            return (board, true);
        }
    }
    board = laymine_op(row, column, minenum, x0, y0);
    (board, false)
}

/// 调整法无猜埋雷。可以生成任意雷密度的无猜局面。但雷不满足均匀分布。  
/// - 输入：高、宽、雷数、起手行数、起手列数  
/// - 返回局面、是否成功  
/// - 性能还有优化的空间。高级局面上埋200雷时，用时大约5秒。  
/// - 用python调用时的示例：  
/// ```python
/// import ms_toollib as ms
/// (board, flag) = laymine_solvable_adjust(16, 30, 200, 3, 20) # flag指示是否成功，极大概率是成功的
/// ```
// 局面中的-10代表还没埋雷。
pub fn laymine_solvable_adjust(
    row: usize,
    column: usize,
    minenum: usize,
    x0: usize,
    y0: usize,
) -> (Vec<Vec<i32>>, bool) {
    // 利用局面调整算法，无猜埋雷
    let mut board;
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
    if row * column - area_op < minenum {
        // 雷数太多以致起手无法开空，此时放弃无猜，返回任意一种局面
        let t = laymine(row, column, minenum, x0, y0);
        if row * column - minenum == 1 {
            return (t, true);
        } else {
            return (t, false);
        }
    }
    if row * column == area_op + minenum {
        return (laymine_op(row, column, minenum, x0, y0), true);
    }

    board = vec![vec![-10; column]; row];
    let mut board_of_game = vec![vec![10; column]; row];
    board_of_game[x0][y0] = 0;
    let remain_minenum = minenum;
    let remain_not_minenum = row * column - area_op - minenum;
    let mut cells_plan_to_click = vec![];
    // 初始化第一步计划点开的格子
    for j in max(1, x0) - 1..min(row, x0 + 2) {
        for k in max(1, y0) - 1..min(column, y0 + 2) {
            board[j][k] = 0;
            if j != x0 || k != y0 {
                cells_plan_to_click.push((j, k));
            }
        }
    }
    // 开始递归求解
    let (mut b, flag) = adjust_step(
        &board,
        &board_of_game,
        &cells_plan_to_click,
        remain_minenum,
        remain_not_minenum,
    );
    // println!("+++++");
    // b.iter().for_each(|i| {
    //     i.iter()
    //         .for_each(|j| print!("{number:>width$}", number = j, width = 4));
    //     println!("")
    // });
    if !flag || b.is_empty() {
        return (laymine_op(row, column, minenum, x0, y0), false);
    }
    for i in 0..row {
        for j in 0..column {
            if b[i][j] == -10 {
                b[i][j] = -1;
            }
        }
    }
    // 最后，算数字
    for i in 0..row {
        for j in 0..column {
            if b[i][j] == -1 {
                for m in max(1, i) - 1..min(row, i + 2) {
                    for n in max(1, j) - 1..min(column, j + 2) {
                        if b[m][n] >= 0 {
                            b[m][n] += 1;
                        }
                    }
                }
            }
        }
    }
    (b, flag)
}

// 调整法的递归部分。注意空间复杂度为局面面积乘求解步数。
// 返回没有计算数字的局面和是否成功。
// println!("rn: {:?}", rn);
// b.iter().for_each(|i| {
//     i.iter()
//         .for_each(|j| print!("{number:>width$}", number = j, width = 4));
//     println!("")
// });
fn adjust_step(
    board: &Vec<Vec<i32>>,            // 当前的board，数字没有计算，只有0，-1，-10
    board_of_game: &Vec<Vec<i32>>,    // 当前的board_of_game，数字没有计算，只有10，1
    plan_click: &Vec<(usize, usize)>, // 当前计划点开的格子，递归部分要保证点开后，局面是有解开的可能的
    remain_minenum: usize,            // 当前还要埋的雷数
    remain_not_minenum: usize,        // 当前还要埋的非雷数
) -> (Vec<Vec<i32>>, bool) {
    let mut b = board.clone(); // 克隆一个board的备份
    let mut bg = board_of_game.clone(); // 克隆一个board_of_game的备份
                                        // let mut pc = plan_click.clone();
    let mut r = remain_minenum; // 克隆一个当前还要埋的雷数的备份
    let mut rn = remain_not_minenum; // 克隆一个当前还要埋的非雷数的备份
    for (x, y) in plan_click.into_iter() {
        bg[*x][*y] = 1;
    } // 点开当前的board_of_game的备份上的计划点开的格子，用1临时表示
      // let b_backup = b.clone(); // 克隆一个board的备份
      // let bg_backup = bg.clone(); // 克隆一个board_of_game的备份

    let (Ases, xses, mut bses) = refresh_matrixses(&bg);
    // 所有分支的前沿都已遍历完成。

    // 前沿全部被雷堵塞，无法继续
    // 只对第一块处理。缺点是内存复杂度上升
    let temp = Ases.get(0);
    let As_0;
    match temp {
        Some(val) => As_0 = val,
        None => return (vec![], false),
    };
    let xs_0 = xses.get(0).unwrap();
    let bs_0 = bses.get_mut(0).unwrap(); // 此时这个b向量是完全错误的

    let mut front_xs_0 = xs_0.clone();
    front_xs_0
        .iter_mut()
        .for_each(|p| p.retain(|x| b[x.0][x.1] == -10));
    if front_xs_0[0].is_empty() {
        if rn > 0 {
            return (vec![], false);
        } else {
            return (b, true);
        }
    }
    // 前沿格子————此格子在边缘，且是没有埋过雷的
    let xs_cell_num = front_xs_0.iter().fold(0, |acc, x| acc + x.len());
    let minenum_except = (xs_cell_num as f64 * r as f64 / (rn + r) as f64) as usize;
    // let mut success_flag = false;
    // 对不同雷数循环
    'inner: for i in 0..xs_cell_num + 1 {
        // 根据算法的映射，计算出minenum，计划的埋雷量
        let minenum;
        if minenum_except == 0 || xs_cell_num == 0 {
            minenum = 0;
        } else if minenum_except == xs_cell_num {
            minenum = minenum_except;
        } else if i == xs_cell_num - 1 {
            minenum = 0;
        } else if i == xs_cell_num {
            minenum = xs_cell_num;
        } else if xs_cell_num > minenum_except * 2 {
            let z = minenum_except * 2 - 1;
            if i < z {
                if i % 2 == 1 {
                    minenum = minenum_except + (i + 1) / 2;
                } else {
                    minenum = minenum_except - i / 2;
                }
            } else {
                minenum = i + 1;
            }
        } else {
            let z = (xs_cell_num - minenum_except) * 2 - 1;
            if i < z {
                if i % 2 == 1 {
                    minenum = minenum_except + (i + 1) / 2;
                } else {
                    minenum = minenum_except - i / 2;
                }
            } else {
                minenum = xs_cell_num - i - 1;
            }
        }
        // 排除显而易见不可能的情况。
        if minenum > r || xs_cell_num - minenum > rn {
            continue;
        }
        // 对每种雷数，重复尝试5次。
        for _u in 0..3 {
            adjust_the_area_on_board(&mut b, &front_xs_0, minenum);
            // 以下的循环用来修正b向量
            for bb in 0..bs_0.len() {
                for ss in 0..bs_0[bb].len() {
                    // ss是第几个方程
                    bs_0[bb][ss] = 0;
                    for aa in 0..As_0[bb][0].len() {
                        // aa是第几个格子
                        if As_0[bb][ss][aa] == 1 {
                            if b[xs_0[bb][aa].0][xs_0[bb][aa].1] == -1
                                && bg[xs_0[bb][aa].0][xs_0[bb][aa].1] != 11
                            {
                                bs_0[bb][ss] += 1;
                            }
                        }
                    }
                }
            }
            let (n, i) = get_all_not_and_is_mine_on_board(
                &mut As_0.clone(),
                &mut xs_0.clone(),
                &mut bs_0.clone(),
                &mut bg,
            );
            i.iter().for_each(|x| bg[x.0][x.1] = 10); // get_all_not_and_is_mine_on_board是修改局面的，修回来

            if n.len() > 0 {
                n.iter().for_each(|x| bg[x.0][x.1] = 1);
                i.iter().for_each(|x| bg[x.0][x.1] = 11);
                // println!("当前步骤成功！");
                // success_flag = true; // 当前步骤成功
                if rn <= n.len() {
                    return (b.clone(), true);
                }
                // pc.append(&mut n);
                r -= minenum;
                rn -= xs_cell_num - minenum;
                let a = adjust_step(&b, &bg, &n, r, rn);
                if a.1 {
                    if !a.0.is_empty() {
                        return (a.0, true);
                    }
                } else {
                    b = board.clone();
                    bg = board_of_game.clone();
                    r = remain_minenum;
                    rn = remain_not_minenum;
                    // continue 'inner;
                    continue;
                }
            }
        }
    }
    return (vec![], false);
}

// 在指定的局部（area_current_adjust）埋雷，不刷新board上的数字
fn adjust_the_area_on_board(
    board: &mut Vec<Vec<i32>>,
    area_current_adjust: &Vec<Vec<(usize, usize)>>,
    minenum: usize,
) {
    // let row = board.len();
    // let column = board[0].len();
    let cell_num = area_current_adjust.iter().fold(0, |acc, x| acc + x.len());
    let mut b = vec![0; cell_num - minenum];
    b.append(&mut vec![-1; minenum]);

    #[cfg(any(feature = "py", feature = "rs"))]
    let mut rng = thread_rng();
    // let mut rng = StdRng::seed_from_u64(532);
    #[cfg(any(feature = "py", feature = "rs"))]
    b.shuffle(&mut rng);

    #[cfg(feature = "js")]
    b.shuffle_();

    let mut id = 0;
    for i in area_current_adjust {
        for &(x, y) in i {
            board[x][y] = b[id];
            id += 1;
        }
    }
}

/// 埋雷并计算高级局面3BV的引擎，用于研究高级3BV的分布。16线程。传入局数，例如1000 000。试一下你的电脑算的有多块吧。  
#[cfg(any(feature = "py", feature = "rs"))]
pub fn sample_3BVs_exp(x0: usize, y0: usize, n: usize) -> [usize; 382] {
    // 从标准高级中采样计算3BV
    // 16线程计算
    let n0 = n / 16;
    let mut threads = vec![];
    for _i in 0..16 {
        let join_item = thread::spawn(move || -> [usize; 382] { laymine_study_exp(x0, y0, n0) });
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
fn laymine_study_exp(x0: usize, y0: usize, n: usize) -> [usize; 382] {
    let mut rng = thread_rng();
    // let area: usize = 16 * 30 - 1;
    let pointer = x0 + y0 * 16;
    let mut bv_record = [0; 382];
    for _id in 0..n {
        let mut board1_dim = [0; 479];
        for i in 380..479 {
            board1_dim[i] = -1;
        }

        board1_dim.shuffle(&mut rng);
        let mut board1_dim_2 = [0; 480];
        // Board1Dim_2.reserve(area + 1);

        for i in 0..pointer {
            board1_dim_2[i] = board1_dim[i];
        }
        board1_dim_2[pointer] = 0;
        for i in pointer..479 {
            board1_dim_2[i + 1] = board1_dim[i];
        }
        let mut board: Vec<Vec<i32>> = vec![vec![0; 30]; 16];
        for i in 0..480 {
            if board1_dim_2[i] < 0 {
                let x = i % 16;
                let y = i / 16;
                board[x][y] = -1;
                for j in max(1, x) - 1..min(16, x + 2) {
                    for k in max(1, y) - 1..min(30, y + 2) {
                        if board[j][k] >= 0 {
                            board[j][k] += 1;
                        }
                    }
                }
            }
        }
        bv_record[cal_bbbv_exp(&board)] += 1;
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
/// - 性能：识别的成功率不是百分之百。识别失败时，甚至可能出现不可恢复的错误。想提高成功率，需要满足：图片清晰、格子的宽度在8到300像素之间、图片中没有鼠标遮挡。  
/// - 以下提供一段用python调用时的示例：  
/// ```python
/// # pip install ms_toollib==1.3.6（windows和linux不一样，请查看[主页](https://github.com/eee555/ms_toollib)）
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

// 扫雷AI
#[cfg(any(feature = "py", feature = "rs"))]
pub fn agent_step(board_of_game: Vec<Vec<i32>>, _pos: (usize, usize)) -> Result<usize, String> {
    let _board_of_game_input: Vec<Vec<f32>> = board_of_game
        .into_iter()
        .map(|x| x.into_iter().map(|y| y as f32).collect::<Vec<f32>>())
        .collect_vec();
    let model = (tract_onnx::onnx()
        .model_for_path("ppo_agent.onnx")
        .unwrap()
        .with_input_fact(
            0,
            InferenceFact::dt_shape(f32::datum_type(), tvec!(1i32, 16, 30)),
        )
        .unwrap()
        .with_input_fact(
            1,
            InferenceFact::dt_shape(f32::datum_type(), tvec!(1i32, 2)),
        )
        .unwrap()
        .into_optimized()
        .unwrap()
        .into_runnable())
    .unwrap();

    let cell_image = vec![10f32; 480];
    let image: Tensor = Array::from_shape_vec((1, 16, 30), cell_image.clone())
        .unwrap()
        .into();
    let image_2: Tensor = Array::from_shape_vec((1, 2), vec![0f32; 2]).unwrap().into();
    let ans = model.run(tvec!(image, image_2)).unwrap();
    let _aaa = ans[0].to_array_view::<i32>().unwrap();
    // println!("{:?}", aaa);
    Ok(30)
}

/// 对局面用单集合、双集合判雷引擎，快速标雷、标非雷，以供概率计算引擎处理。这是非常重要的加速。  
/// 相当于一种预处理，即先标出容易计算的。mark可能因为无解而报错，此时返回错误码。  
/// 若不合法，直接中断，不继续标记。  
/// - 注意：在rust中，cal_possibility往往需要和mark_board搭配使用，而在其他语言（python）中可能不需要如此！这是由于其ffi不支持原地操作。
pub fn mark_board(board_of_game: &mut Vec<Vec<i32>>) -> Result<(), usize> {
    let (mut As, mut xs, mut bs, _, _) = refresh_matrixs(&board_of_game);
    solve_direct(&mut As, &mut xs, &mut bs, board_of_game)?;
    for i in 0..As.len() {
        for j in 0..As[i].len() {
            if As[i][j].iter().sum::<i32>() < bs[i][j] || bs[i][j] < 0 {
                return Err(7);
            }
        }
    }
    let _ = solve_minus(&mut As, &mut xs, &mut bs, board_of_game);
    for i in 0..As.len() {
        for j in 0..As[i].len() {
            if As[i][j].iter().sum::<i32>() < bs[i][j] || bs[i][j] < 0 {
                return Err(8);
            }
        }
    }
    Ok(())
}

/// 求出游戏局面中所有非雷、是雷的位置。  
/// - 注意：局面中可以有标雷，但不能有错误！
pub fn get_all_not_and_is_mine_on_board(
    As: &mut Vec<Vec<Vec<i32>>>,
    xs: &mut Vec<Vec<(usize, usize)>>,
    bs: &mut Vec<Vec<i32>>,
    board_of_game: &mut Vec<Vec<i32>>,
) -> (Vec<(usize, usize)>, Vec<(usize, usize)>) {
    let mut ans = solve_direct(As, xs, bs, board_of_game).unwrap();
    let mut is_mine = vec![];
    let mut not_mine = vec![];
    not_mine.append(&mut ans.0);
    is_mine.append(&mut ans.1);
    let mut ans = solve_minus(As, xs, bs, board_of_game).unwrap();
    not_mine.append(&mut ans.0);
    is_mine.append(&mut ans.1);
    let mut ans = solve_enumerate(As, xs, bs);
    not_mine.append(&mut ans.0);
    is_mine.append(&mut ans.1);
    (not_mine, is_mine)
}

/// 判断是否为可能可以（区别于必然可以）判雷时的猜雷；
/// 对应弱无猜、准无猜规则。
/// - 前提：点在未知格上，即10。  
/// - 约定：1 -> 正确的判雷。  
///  - 注意：不可以处理14、15等标记（全当成10）。输入为玩家维护的游戏局面，因此会首先清干净玩家标的雷。  
/// 2 -> 必要的猜雷。（由于全局或局部不可判而猜雷）  
/// 3 -> 不必要的猜雷。  
/// 4 -> 踩到必然的雷。  
/// 5 -> 没有结果。因为此处已经被点开了。
pub fn is_guess_while_needless(board_of_game: &mut Vec<Vec<i32>>, xy: &(usize, usize)) -> i32 {
    board_of_game.iter_mut().for_each(|x| {
        x.iter_mut().for_each(|xx| {
            if *xx > 10 {
                *xx = 10
            }
        })
    });
    if board_of_game[xy.0][xy.1] < 10 {
        return 5;
    }
    let mut flag_need;
    let (mut Ases, mut xses, mut bses) = refresh_matrixses(&board_of_game);
    if let (Some(xy), flag_border) = find_a_border_cell(board_of_game, xy) {
        let t = xses
            .iter()
            .position(|r| r.iter().any(|x| x.contains(&xy)))
            .unwrap();
        let As = &mut Ases[t];
        let xs = &mut xses[t];
        let bs = &mut bses[t];
        let (n, _) = solve_direct(As, xs, bs, board_of_game).unwrap();
        if !flag_border && !n.is_empty() {
            return 3;
        }
        flag_need = n.is_empty();
        match board_of_game[xy.0][xy.1] {
            12 => return 1,
            11 => return 4,
            _ => {
                let (n, _) = solve_minus(As, xs, bs, board_of_game).unwrap();
                if !flag_border && !n.is_empty() {
                    return 3;
                }
                flag_need = flag_need && n.is_empty();
                match board_of_game[xy.0][xy.1] {
                    12 => return 1,
                    11 => return 4,
                    _ => {
                        let (n, i) = solve_enumerate(As, xs, bs);
                        if !flag_border && !n.is_empty() {
                            return 3;
                        }
                        flag_need = flag_need && n.is_empty();
                        if n.contains(&xy) {
                            return 1;
                        } else if i.contains(&xy) {
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
    } else {
        return 2; // 无论何时，包心雷，是合理的猜雷。
    }
}

/// 判断是否为判雷；对应强无猜规则。
/// - 前提：对打开非雷或标记是雷的行为判断。   
/// - 不仅可以判断是雷，也可以判断非雷。  
/// - 注意：不可以处理14、15等标记（当成10处理）。输入为玩家维护的游戏局面，因此会首先清干净玩家标的雷。  
pub fn is_able_to_solve(board_of_game: &mut Vec<Vec<i32>>, xy: &(usize, usize)) -> bool {
    board_of_game.iter_mut().for_each(|x| {
        x.iter_mut().for_each(|xx| {
            if *xx > 10 {
                *xx = 10
            }
        })
    });
    let (mut As, mut xs, mut bs, _, _) = refresh_matrixs(&board_of_game);
    let _ = solve_direct(&mut As, &mut xs, &mut bs, board_of_game);
    if board_of_game[xy.0][xy.1] == 11 || board_of_game[xy.0][xy.1] == 12 {
        return true;
    }
    let _ = solve_minus(&mut As, &mut xs, &mut bs, board_of_game);
    if board_of_game[xy.0][xy.1] == 11 || board_of_game[xy.0][xy.1] == 12 {
        return true;
    }
    let (n, i) = solve_enumerate(&As, &xs, &bs);
    if i.contains(xy) || n.contains(xy) {
        return true;
    }
    false
}
