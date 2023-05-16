use ms_toollib::{
    cal_is_op_possibility_cells, cal_possibility, cal_possibility_onboard, is_solvable, mark_board,
    solve_direct, solve_enumerate,
};
use ms_toollib::{
    cal_table_minenum_recursion, combine, refresh_matrix, refresh_matrixs, refresh_matrixses,
};

// 测试获取局面矩阵的函数

#[test]
fn refresh_matrixses_works() {
    // 测试获取局面矩阵引擎
    let mut game_board = vec![
        vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        vec![
            1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        vec![
            10, 10, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0,
        ],
        vec![
            10, 10, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0,
        ],
        vec![
            2, 3, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        vec![
            10, 10, 11, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0,
        ],
        vec![
            10, 10, 10, 2, 0, 0, 1, 1, 2, 1, 2, 2, 3, 3, 2, 2, 2, 3, 3, 2, 2, 2, 3, 3, 2, 2, 1, 1,
            0, 0,
        ],
        vec![
            10, 10, 10, 1, 0, 0, 1, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 11, 11, 10, 10, 10, 11,
            11, 10, 10, 10, 1, 0, 0,
        ],
        vec![
            10, 10, 10, 1, 0, 0, 1, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 1, 0, 0,
        ],
    ];
    let (matrix_ases, matrix_xses, matrix_bses) = refresh_matrixses(&game_board);
    println!("matrix_ases: {:?}", matrix_ases);
    println!("matrix_xses: {:?}", matrix_xses);
    println!("matrix_bses: {:?}", matrix_bses);
}

#[test]
fn refresh_matrixses_works2() {
    // 测试获取局面矩阵引擎
    let game_board = vec![
        vec![10, 0, 0, 0, 1, 10, 10],
        vec![10, 0, 0, 0, 1, 10, 10],
        vec![10, 0, 0, 0, 2, 10, 10],
        vec![0, 0, 0, 0, 1, 10, 10],
        vec![0, 0, 0, 0, 1, 10, 10],
    ];
    let (matrix_ases, matrix_xses, matrix_bses) = refresh_matrixses(&game_board);
    println!("matrix_ases: {:?}", matrix_ases);
    println!("matrix_xses: {:?}", matrix_xses);
    println!("matrix_bses: {:?}", matrix_bses);
    println!("matrix_bses: {:?}", cal_possibility_onboard(&game_board, 4.0));
}
