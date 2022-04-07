// 测试录像分析模块
use ms_toollib::{AvfVideo, MinesweeperBoard};

#[test]
fn minesweeper_board_works() {
    // 局面状态机测试
    // [("lc", (4, 4)), ("lr", (4, 4)), ("rc", (2, 6)), ("rr", (2, 6)), ("lc", (3, 6)), ("cc", (3, 6)), ("rr", (3, 6))]
    let board = vec![
        vec![-1, -1, 2, 1, 0, 0, 1, -1],
        vec![-1, 5, -1, 1, 0, 0, 2, 2],
        vec![-1, 3, 1, 1, 0, 1, 3, -1],
        vec![1, 1, 0, 0, 0, 1, -1, -1],
        vec![0, 0, 0, 0, 0, -1, 2, 2],
        vec![0, 0, 0, 0, 1, 1, 1, 0],
        vec![0, 0, 0, 0, 1, -1, 1, 0],
        vec![0, 0, 0, 0, 1, 1, 1, 0],
    ];
    let mut my_board = MinesweeperBoard::new(board);
    my_board
        .step_flow(vec![
            ("lc", (4, 5)),
            ("lr", (4, 5)),
            ("rc", (0, 4)),
            ("rr", (0, 4)),
            ("lc", (0, 3)),
            ("cc", (0, 3)),
            ("rr", (0, 3)),
            ("lr", (0, 3)),
            ("lc", (2, 4)),
            ("lr", (2, 4)),
            ("rc", (4, 6)),
            ("rr", (4, 6)),
            ("lc", (4, 5)),
            ("cc", (4, 5)),
            ("rr", (4, 5)),
            ("lr", (4, 5)),
        ])
        .unwrap();
    my_board.board.iter().for_each(|x| println!("{:?}", x));
    my_board.game_board.iter().for_each(|x| println!("{:?}", x));
    println!("{:?}", my_board.game_board_state);
}

#[test]
// cargo test --features rs -- --nocapture AvfVideo_works
fn AvfVideo_works() {
    // 录像解析工具测试
    let mut video = AvfVideo::new("i.avf");
    let r = video.parse_video();
    println!("结果：{:?}", r);
    println!("标识：{:?}", video.player);
    println!("3BV：{:?}", video.static_params.bbbv);
    // video.print_event();
    video.analyse();
}

#[test]
fn temp() {
    let mut board = vec![
        vec![-1, 8, 2, 1, 0, 0, 1, -1],
        vec![-1, 5, -8, 1, 0, 0, 2, 2],
        vec![-1, 3, 1, 1, 888, 1, 3, -1],
        vec![1, 1, 0, 0, 0, 1, -1, -1],
        vec![0, 777, 0, 0, 0, -1, 2, 2],
        vec![0, 0, 0, 0, 1, 1, 1, 0],
        vec![0, 0, 0, 0, 999, -1, 1, 0],
        vec![0, 0, 0, 0, 1, 1, 1, 0],
    ];
    board.iter_mut().for_each(|x| x.iter_mut().for_each(|xx| if *xx > 10 { *xx = 10 }));
    println!("{:?}", board);
}



