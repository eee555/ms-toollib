use ms_toollib::{cal_bbbv, cal_op, laymine};

#[test]
fn cal_bbbv_works() {
    let game_board = vec![
        vec![1, 1, 0, 0, 0, 0, 0, 0],
        vec![-1, 2, 0, 0, 0, 0, 0, 0],
        vec![-1, 2, 0, 0, 0, 1, 1, 1],
        vec![2, 2, 1, 0, 0, 1, -1, 1],
        vec![1, -1, 2, 1, 1, 2, 2, 2],
        vec![2, 3, 5, -1, 2, 1, -1, 2],
        vec![1, -1, -1, -1, 2, 1, 2, -1],
        vec![1, 2, 3, 2, 1, 0, 1, 1],
    ]; // 20
    let bbbv = cal_bbbv(&game_board);
    print!("bbbv: {:?}", bbbv);
    let op = cal_op(&game_board);
    print!("op: {:?}", op);
}
