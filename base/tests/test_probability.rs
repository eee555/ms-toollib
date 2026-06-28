use ms_toollib::{
    cal_all_solution, cal_probability_cells_is_op, cal_probability_csp, cal_probability_enum,
    cal_probability_onboard,
};

#[test]
fn board_1_works() {
    let board = vec![
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 1, 10, 10, 10, 10],
        vec![10, 10, 10, 2, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 8, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
    ];
    let ans = cal_probability_csp(&board, 10.0).unwrap();
    println!("{:?}", ans);
    assert_eq!(ans.2, [10, 10, 54]);
    let ans = cal_probability_enum(&board, 10.0).unwrap();
    println!("{:?}", ans);
    assert_eq!(ans.2, [10, 10, 54]);
}

#[test]
fn board_2_works() {
    let mut board = vec![
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
    ];
    let ans = cal_probability_csp(&board, 10.0).unwrap();
    println!("{:?}", ans);
    assert_eq!(ans.2, [0, 10, 64]);
    let ans = cal_probability_enum(&board, 10.0).unwrap();
    println!("{:?}", ans);
    assert_eq!(ans.2, [0, 10, 64]);
}


#[test]
fn board_3_works() {
    let mut board = vec![
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 2, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
    ];
    let ans = cal_probability_csp(&board, 10.0).unwrap();
    println!("{:?}", ans);
    assert_eq!(ans.2, [2, 10, 57]);
    let ans = cal_probability_csp(&board, 0.0).unwrap();
    println!("{:?}", ans);
    assert_eq!(ans.2, [2, 2, 57]);
    let ans = cal_probability_csp(&board, 60.0).unwrap();
    println!("{:?}", ans);
    assert_eq!(ans.2, [2, 57, 57]);
    let ans = cal_probability_enum(&board, 10.0).unwrap();
    println!("{:?}", ans);
    assert_eq!(ans.2, [2, 10, 57]);
}

#[test]
fn board_4_works() {
    let mut board = vec![
        vec![1, 10, 10, 10, 10, 10, 10, 10],
        vec![1, 10, 10, 10, 10, 10, 10, 10],
        vec![2, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
        vec![10, 10, 10, 10, 10, 10, 10, 10],
    ];
    let ans = cal_probability_csp(&board, 10.0).unwrap();
    println!("{:?}", ans);
    assert_eq!(ans.2, [2, 10, 59]);
    assert_eq!(ans.1, 0.14150943396226415);
    for (cell, p) in ans.0 {
        if cell == (0, 1) {
            assert_eq!(p, 0.07547169811320754)
        }
        if cell == (1, 1) {
            assert_eq!(p, 0.9245283018867925)
        }
        if cell == (2, 0) {
            assert_eq!(p, 0.0)
        }
        if cell == (3, 0) {
            assert_eq!(p, 0.5377358490566039)
        }
        if cell == (3, 1) {
            assert_eq!(p, 0.5377358490566039)
        }
    }
    let ans = cal_probability_enum(&board, 10.0).unwrap();
    println!("{:?}", ans);
    assert_eq!(ans.2, [2, 10, 59]);
    assert_eq!(ans.1, 0.14150943396226415);
    for (cell, p) in ans.0 {
        if cell == (0, 1) {
            assert_eq!(p, 0.07547169811320754)
        }
        if cell == (1, 1) {
            assert_eq!(p, 0.9245283018867925)
        }
        if cell == (2, 0) {
            assert_eq!(p, 0.0)
        }
        if cell == (3, 0) {
            assert_eq!(p, 0.5377358490566039)
        }
        if cell == (3, 1) {
            assert_eq!(p, 0.5377358490566039)
        }
    }
}
