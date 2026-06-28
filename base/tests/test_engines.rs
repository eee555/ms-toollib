use ms_toollib::{
    cal_all_solution, cal_probability_cells_is_op, cal_probability_csp, cal_probability_enum,
    cal_probability_onboard, is_able_to_solve, is_guess_while_needless, mark_board, solve_direct,
    solve_enumerate, try_solve,
};
use ms_toollib::{cal_bbbv, cal_table_minenum_recursion, combine, refresh_matrix, refresh_matrixs};

// 测试各种引擎类的函数

#[test]
fn cal_is_op_probability_cells_works() {
    // 测试开空概率计算函数
    let game_board = vec![
        vec![10, 11, 1, 1, 10, 1, 0, 0],
        vec![10, 10, 1, 10, 10, 3, 2, 1],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 2, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 12, 10, 10],
    ];
    let ans = cal_probability_cells_is_op(
        &game_board,
        20,
        &vec![(0, 0), (1, 1), (1, 6), (7, 2), (7, 5)],
    );
    assert_eq!(ans[0], 0.0);
    assert_eq!(ans[1], 0.0);
    assert_eq!(ans[2], 0.0);
    assert_eq!(ans[3], 0.0);
    assert_eq!(ans[4], 0.0);
}

#[test]
fn solve_direct_works() {
    // 测试枚举判雷引擎
    let mut game_board = vec![
        vec![
            10, 10, 10, 1, 1, 0, 0, 1, 11, 1, 0, 0, 0, 0, 1, 10, 10, 10, 2, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 11, 2, 0, 0, 1, 1, 1, 1, 2, 2, 1, 1, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 11, 4, 2, 1, 0, 0, 0, 1, 11, 11, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 10, 11, 11, 2, 1, 0, 0, 2, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 10, 10, 10, 11, 3, 1, 1, 1, 11, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10, 10,
        ],
    ];
    let (mut matrix_as, mut matrix_xs, mut matrix_bs, _, _) = refresh_matrixs(&game_board);
    let ans = solve_direct(
        &mut matrix_as,
        &mut matrix_xs,
        &mut matrix_bs,
        &mut game_board,
    );
    print!("{:?}", ans)
}

#[test]
fn solve_enumerate_works() {
    // 测试枚举判雷引擎
    let game_board = vec![
        vec![0, 0, 1, 10, 10, 10, 10, 10],
        vec![0, 0, 2, 10, 10, 10, 10, 10],
        vec![1, 1, 3, 11, 10, 10, 10, 10],
        vec![10, 10, 4, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
    ];
    let (matrix_as, matrix_xs, matrix_bs, _, _) = refresh_matrixs(&game_board);
    let ans = solve_enumerate(&matrix_as, &matrix_xs, &matrix_bs);
    print!("{:?}", ans)
}

#[test]
fn cal_probability_onboard_1_works() {
    // 测试概率计算引擎
    let mut game_board = vec![
        vec![10, 10, 1, 1, 10, 1, 0, 0],
        vec![10, 10, 1, 10, 10, 3, 2, 1],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 2, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
    ];
    let ans = cal_probability_enum(&game_board, 10.0);
    print!("设置雷数为10，概率计算引擎的结果为：{:?}", ans);
    let ans = cal_probability_enum(&game_board, 0.15625);
    print!("设置雷的比例为15.625%，概率计算引擎的结果为：{:?}", ans);
    // 对局面预标记，以加速计算
    let _ = mark_board(&mut game_board, false);
    let ans = cal_probability_onboard(&game_board, 10.0);
    print!("设置雷的比例为10，与局面位置对应的概率结果为：{:?}", ans);
}

#[test]
fn cal_probability_onboard_2_works() {
    // 测试概率计算引擎
    let game_board = vec![
        vec![
            10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 3, 10, 3, 10, 2, 10, 2, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 10, 10, 10, 10, 10, 2, 10, 10, 10, 10, 10, 1, 10, 1, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 10, 10, 10, 2, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 2,
            1, 2, 2, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 10, 10, 1, 10, 10, 10, 10, 10, 10, 10, 4, 10, 10, 10, 10, 10, 10, 10, 10,
            1, 0, 1, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 2, 1, 1, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 1,
            0, 1, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 1, 1, 1, 0, 1, 10, 10, 10, 10, 10, 3, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 1, 0,
            2, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 1, 0, 0, 1, 3, 10, 10, 10, 2, 10, 10, 10, 10, 10, 10, 10, 10, 2, 10, 10, 10, 1, 1,
            2, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 2, 1, 0, 1, 10, 10, 3, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 2, 10, 10,
            10, 10, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 1, 0, 1, 10, 10, 10, 10, 10, 10, 1, 10, 1, 10, 10, 10, 10, 10, 1, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10,
        ],
        vec![
            1, 1, 1, 0, 1, 1, 10, 10, 10, 1, 10, 10, 10, 10, 10, 10, 10, 10, 4, 10, 10, 1, 10, 10,
            10, 10, 10, 10, 10, 10,
        ],
        vec![
            0, 0, 0, 0, 1, 10, 10, 10, 10, 10, 10, 10, 10, 10, 1, 10, 10, 3, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10,
        ],
        vec![
            0, 1, 1, 1, 1, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10,
        ],
        vec![
            0, 1, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10, 10,
        ],
        vec![
            0, 1, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10, 10,
        ],
    ];
    let ans = cal_probability_enum(&game_board, 99.0);
    // let ans = cal_probability_enum(&game_board, 0.15625);
    print!("{:?}", ans)
}

#[test]
fn cal_probability_onboard_3_works() {
    // 测试概率计算引擎
    let mut game_board = vec![
        vec![1, 1, 2, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10],
    ];
    let _ = mark_board(&mut game_board, false);

    let (board_poss, [mine_min, mine_num, mine_max]) =
        cal_probability_onboard(&game_board, 10.0).unwrap();
    assert_eq!(board_poss[0][3], 0.5444444444444445);
    assert_eq!(board_poss[1][0], 0.0888888888888889);
    assert_eq!(board_poss[1][1], 0.9111111111111113);
    assert_eq!(board_poss[1][2], 0.0);
    assert_eq!(board_poss[1][3], 0.5444444444444445);
    assert_eq!(mine_min, 2);
    assert_eq!(mine_num, 10);
    assert_eq!(mine_max, 51);
}

#[test]
fn cal_probability_onboard_4_works() {
    // 测试概率计算引擎
    let game_board = vec![
        vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        vec![
            0, 0, 0, 0, 0, 0, 0, 1, 1, 3, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        vec![
            0, 0, 0, 0, 0, 0, 0, 1, 10, 0, 1, 0, 0, 10, 0, 0, 0, 10, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
    ];
    let ans = cal_probability_enum(&game_board, 0.0);
    print!("{:?}", ans)
}

#[test]
fn cal_probability_onboard_5_works() {
    // 测试概率计算引擎
    let mut game_board = vec![
        vec![
            10, 10, 10, 2, 0, 0, 0, 0, 1, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 3, 1, 0, 0, 0, 1, 1, 2, 1, 2, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 10, 1, 0, 0, 0, 0, 0, 0, 0, 1, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 10, 2, 0, 0, 0, 0, 0, 0, 0, 2, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 10, 1, 0, 0, 0, 0, 0, 0, 1, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 10, 1, 1, 2, 2, 1, 0, 0, 1, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 10, 10, 10, 10, 10, 2, 1, 0, 1, 2, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 10, 10, 10, 10, 10, 10, 1, 0, 0, 1, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 10, 10, 10, 10, 2, 1, 1, 0, 1, 1, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 10, 10, 10, 10, 1, 0, 0, 0, 2, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 10, 10, 10, 10, 3, 2, 2, 1, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10, 10,
        ],
        vec![
            10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10, 10,
        ],
    ];
    // let b = refresh_matrixs(&game_board);
    let b = mark_board(&mut game_board, false);
    // let ans = cal_probability_onboard(&game_board, 10.0);
    // let (mut matrix_as, mut matrix_xs, mut matrix_bs, _, _) = refresh_matrixs(&game_board);
    // let ans = solve_direct(
    //     &mut matrix_as,
    //     &mut matrix_xs,
    //     &mut matrix_bs,
    //     &mut game_board,
    // );
    // print!("{:?}, {:?}, {:?}", matrix_as, matrix_xs, matrix_bs)
    print!("{:?}", b)
}

#[test]
fn cal_probability_onboard_7_works() {
    // 测试概率计算引擎
    let mut game_board = vec![
        vec![10, 1, 0, 0, 0, 0, 0, 0],
        vec![10, 1, 0, 0, 0, 0, 0, 0],
        vec![10, 1, 0, 0, 0, 1, 1, 1],
        vec![10, 1, 1, 1, 1, 2, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 7, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
    ];
    let _ = mark_board(&mut game_board, true);
    // let ans = cal_probability_onboard(&game_board, 30.0);
    // print!("{:?}", ans);
}

#[test]
fn cal_table_minenum_recursion_works() {
    // 测试递归枚举引擎
    //     [[0, 0, 1, -1, 2, 1, 1, -1], [0, 0, 2, 3, -1, 3, 3, 2], [1, 1, 3, -1, 4, -1, -1, 2], [2, -1, 4, -1, 3, 4, -1, 4], [3, -1, 5, 2, 1, 3, -1, -1], [3, -1, -1, 2, 1, 2, -1, 3], [-1, 5, 4, -1,
    // 1, 1, 2, 2], [-1, 3, -1, 2, 1, 0, 1, -1]]
    let game_board = vec![
        vec![0, 0, 1, 10, 10, 10, 10, 10],
        vec![0, 0, 2, 10, 10, 10, 10, 10],
        vec![1, 1, 3, 11, 10, 10, 10, 10],
        vec![10, 10, 4, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
    ];
    // let a = isSolvable(&board, 0, 0, 40);
    // print!("{:?}", a);
    let (matrix_a, matrix_x, matrix_b) = refresh_matrix(&game_board);
    let (matrix_a_s, matrix_x_s, combination_relationship) = combine(&matrix_a, &matrix_x);
    let table = cal_table_minenum_recursion(
        &matrix_a_s,
        &matrix_x_s,
        &matrix_b,
        &combination_relationship,
    );

    let a = cal_all_solution(&matrix_a, &matrix_b);
    println!("table的结果为：{:?}", table);
    println!("matrix_a的结果为：{:?}", matrix_a);
    println!("matrix_x的结果为：{:?}", matrix_x);
    println!("matrix_b的结果为：{:?}", matrix_b);
    println!("a的结果为：{:?}", a);
}

#[test]
fn is_guess_while_needless_works() {
    // let mut game_board = vec![
    //     vec![0, 0, 1, 10, 10, 10, 10, 10],
    //     vec![0, 0, 2, 10, 10, 10, 10, 10],
    //     vec![1, 1, 3, 11, 10, 10, 10, 10],
    //     vec![10, 10, 4, 10, 10, 10, 10, 10],
    //     vec![10, 10, 10, 10, 10, 10, 10, 10],
    //     vec![10, 10, 10, 10, 10, 10, 10, 10],
    //     vec![10, 10, 10, 10, 10, 10, 10, 10],
    //     vec![10, 10, 10, 10, 10, 10, 10, 10],
    // ];
    let mut game_board = vec![
        vec![1, 10, 10, 2, 10, 2, 1, 0],
        vec![10, 10, 10, 10, 10, 10, 1, 0],
        vec![10, 10, 10, 10, 10, 3, 1, 0],
        vec![10, 10, 10, 10, 1, 1, 0, 0],
        vec![10, 10, 10, 10, 1, 0, 0, 0],
        vec![10, 10, 10, 10, 1, 0, 0, 0],
        vec![10, 10, 10, 10, 2, 1, 0, 0],
        vec![10, 10, 10, 10, 10, 1, 0, 0],
    ];
    let code = is_guess_while_needless(&mut game_board, &(3, 2));
    println!("{:?}", code);
    let code = is_guess_while_needless(&mut game_board, &(0, 1));
    println!("{:?}", code);
    let code = is_guess_while_needless(&mut game_board, &(0, 4));
    println!("{:?}", code);
    // let code = is_guess_while_needless(&mut game_board, &(0, 3));
    // println!("{:?}", code);

    let mut game_board = vec![
        vec![0, 0, 1, 1, 1, 0],
        vec![0, 0, 1, 10, 2, 1],
        vec![0, 0, 2, 3, 10, 10],
        vec![0, 0, 1, 10, 10, 10],
        vec![0, 0, 2, 3, 10, 10],
        vec![0, 0, 1, 10, 2, 1],
        vec![0, 0, 1, 1, 1, 0],
    ];
    let code = is_guess_while_needless(&mut game_board, &(2, 4));
    println!("{:?}", code);
}

#[test]
fn is_able_to_solve_works() {
    let mut game_board = vec![
        vec![11, 10, 10, 10, 1, 0, 1, 10],
        vec![11, 10, 2, 2, 1, 0, 2, 10],
        vec![11, 10, 1, 0, 0, 0, 2, 10],
        vec![11, 11, 1, 0, 0, 0, 1, 10],
        vec![1, 1, 1, 1, 2, 2, 1, 10],
        vec![0, 0, 0, 10, 10, 10, 10, 10],
        vec![0, 0, 0, 1, 2, 2, 2, 10],
        vec![0, 0, 0, 0, 0, 0, 1, 10],
    ];
    let code = is_able_to_solve(&mut game_board, &(4, 3));
    println!("{:?}", code);
}

#[test]
fn try_solve_works() {
    let board = vec![
        vec![1, 1, 1, 1, 1, 2, 2, 2],
        vec![1, -1, 1, 2, -1, 3, -1, -1],
        vec![1, 1, 1, 3, -1, 5, 3, 3],
        vec![0, 0, 0, 2, -1, 3, -1, 1],
        vec![0, 0, 0, 1, 2, 4, 3, 2],
        vec![0, 0, 0, 0, 1, -1, -1, 2],
        vec![0, 1, 1, 1, 1, 3, -1, 2],
        vec![0, 1, -1, 1, 0, 1, 1, 1],
    ];
    let a = try_solve(&board, 3, 0);
    println!("{:?}", a);
    println!("{:?}", cal_bbbv(&board));
}

#[test]
fn test_probability_jsminesweeper_simple() {
    // 2x2 board: '1' with 3 unknowns → each should have 1/3 probability
    let board = vec![vec![10, 10], vec![10, 1]];
    // 1 mine, 3 unknown tiles
    let ans = cal_probability_csp(&board, 1.0);
    println!("simple test result: {:?}", ans);
    assert!(ans.is_ok());
    let (probs, p_unknown, range, _) = ans.unwrap();
    // All 3 unknown tiles are adjacent to the '1', each has same probability
    assert_eq!(probs.len(), 3);
    for &(_, prob) in &probs {
        assert!((prob - (1.0 / 3.0)).abs() < 1e-10);
    }
    println!(
        "simple test: probs={:?}, p_unknown={}, range={:?}",
        probs, p_unknown, range
    );
}

#[test]
fn test_probability_jsminesweeper_contradiction() {
    // A '1' with only one adjacent unknown but needs 2 mines → contradiction
    // Create a contradiction: two number tiles sharing the same unknowns
    // with different mine counts that cannot both be satisfied.
    // 2x2 board:
    // [0, 10]
    // [1, 10]
    // Cell (0,0)=0 says 0 mines among (0,1), (1,0), (1,1)
    // Cell (1,0)=1 says 1 mine among (0,0), (0,1), (1,1)
    // Both share unknowns (0,1) and (1,1). Box {min=1, max=0} → impossible.
    let board = vec![vec![0, 10], vec![1, 10]];
    let ans = cal_probability_csp(&board, 1.0);
    assert!(ans.is_err());
    assert_eq!(ans.unwrap_err(), 1);
}

#[test]
fn test_probability_jsminesweeper_known_result() {
    // Simple 2x3 board with a '1' adjacent to 3 unknowns + off-edge tiles
    // [1, 10, 10]
    // [10, 10, 10]
    let board = vec![vec![1, 10, 10], vec![10, 10, 10]];
    let ans = cal_probability_csp(&board, 2.0);
    assert!(ans.is_ok());
    let (probs, p_unknown, range, _) = ans.unwrap();
    // The '1' at (0,0) has 3 unknown neighbors needing 1 mine among them.
    // Probability each: C(1,0)*C(1,1)/C(2,1) = ... actually let's verify numerically.
    // 3 adjoining tiles share 1 mine → 1/3 each.
    // Off-edge: 2 tiles, 1 remaining mine → each off-edge tile 0.5.
    for &(_, prob) in &probs {
        assert!((prob - 1.0 / 3.0).abs() < 1e-10);
    }
    assert!((p_unknown - 0.5).abs() < 1e-10);
    assert_eq!(range, [1, 2, 3]);
}

#[test]
fn test_probability_jsminesweeper_all_unknown() {
    // Board with no numbers - just unknown tiles
    let board = vec![vec![10, 10], vec![10, 10]];
    let ans = cal_probability_csp(&board, 2.0);
    assert!(ans.is_ok());
    let (probs, _p_unknown, _range, _) = ans.unwrap();
    // 2 mines among 4 tiles → each tile has 0.5 probability
    assert_eq!(probs.len(), 4);
    for &(_, prob) in &probs {
        assert!((prob - 0.5).abs() < 1e-10);
    }
}

#[test]
fn test_probability_jsminesweeper_beg() {
    let game_board = vec![
        vec![1, 10, 10, 10, 10, 10, 10, 10],
        vec![1, 10, 10, 10, 10, 10, 10, 10],
        vec![8, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
    ];
    let ans = cal_probability_csp(&game_board, 10.0);
    println!("{:?}", ans);

    let ans = cal_probability_enum(&game_board, 10.0);
    println!("{:?}", ans);
}
