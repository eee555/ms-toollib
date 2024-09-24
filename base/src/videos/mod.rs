pub mod avf_video; 
pub use avf_video::{AvfVideo};
pub mod rmv_video; 
pub use rmv_video::{RmvVideo};
pub mod evf_video; 
pub use evf_video::{EvfVideo};
pub mod mvf_video; 
pub use mvf_video::{MvfVideo};
pub mod base_video; 
pub use base_video::{BaseVideo, valid_time_period};
pub mod minesweeper_board; 
pub use minesweeper_board::{MinesweeperBoard, GameBoardState, MouseState};
mod analyse_methods;


pub trait NewSomeVideo<T> {
    fn new(file_name: T) -> Self;
}

pub trait NewSomeVideo2<T, U> {
    fn new(raw_data: T, file_name: U) -> Self;
}


