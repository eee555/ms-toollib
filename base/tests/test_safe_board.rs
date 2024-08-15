use ms_toollib::{BaseVideo, SafeBoard};
use ms_toollib::videos::base_video::{NewBaseVideo, NewBaseVideo2};

#[test]
fn safe_board_works() {
    let board = vec![
        vec![1, 1, 1, 1, 1, 2, 2, 2],
        vec![1, -1, 1, 2, -1, 3, -1, -1],
        vec![1, 1, 1, 3, -1, 5, 3, 3],
        vec![0, 0, 0, 2, -1, 3, -1, 1],
        vec![0, 0, 0, 1, 1, 3, 2, 2],
        vec![0, 0, 0, 0, 0, 2, -1, 2],
        vec![0, 1, 1, 1, 0, 2, -1, 2],
        vec![0, 1, -1, 1, 0, 1, 1, 1],
    ];
    let mut my_board = SafeBoard::new(board);
    println!("{:?}", my_board.into_vec_vec());
}

#[test]
fn base_video_safe_board_works() {
    let board = vec![
        vec![1, 1, 1, 1, 1, 2, 2, 2],
        vec![1, -1, 1, 2, -1, 3, -1, -1],
        vec![1, 1, 1, 3, -1, 5, 3, 3],
        vec![0, 0, 0, 2, -1, 3, -1, 1],
        vec![0, 0, 0, 1, 1, 3, 2, 2],
        vec![0, 0, 0, 0, 0, 2, -1, 2],
        vec![0, 1, 1, 1, 0, 2, -1, 2],
        vec![0, 1, -1, 1, 0, 1, 1, 1],
    ];
    let mut my_board = BaseVideo::<SafeBoard>::new(board, 16);
    let board = vec![
        vec![1, 1, 1, 0, 0, 0, 1, 1],
        vec![1, -1, 1, 0, 0, 0, 2, -1],
        vec![1, 1, 2, 1, 1, 0, 2, -1],
        vec![0, 1, 2, -1, 1, 1, 2, 2],
        vec![1, 2, -1, 2, 1, 1, -1, 1],
        vec![1, -1, 3, 3, 1, 2, 1, 1],
        vec![2, 3, -1, 2, -1, 1, 0, 0],
        vec![-1, 2, 1, 2, 1, 1, 0, 0],
    ];
    my_board.set_board(board);
    my_board.step("lc", (18,18));
    my_board.step("lr", (18,18));
    println!("{:?}", my_board.minesweeper_board.board.into_vec_vec());
}
