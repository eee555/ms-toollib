use ms_toollib::{cal_bbbv, laymine};


#[test]
fn cal_bbbv_works() {
    let game_board = laymine(16, 30, 99, 0, 0);
    let bbbv = cal_bbbv(&game_board);
    print!("{:?}", bbbv);
}



