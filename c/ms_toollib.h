#include <cstdint>

struct Row {
    int32_t *cells;
    size_t n_column;
};

struct Board {
    struct Row *rows;
    size_t n_row;
};

struct RowPoss {
    double *cells_poss;
    size_t n_column;
};

struct BoardPoss {
    struct RowPoss *rows_poss;
    size_t n_row;
};

struct BoardPossReturn {
    struct BoardPoss board_poss;
    size_t min_mine_num;
    size_t mine_num;
    size_t max_mine_num;
};

enum MouseState {
    UpUp,
    UpDown,
    UpDownNotFlag,
    DownUp,
    Chording,
    ChordingNotFlag,
    DownUpAfterChording,
    Undefined,
};

enum GameBoardState {
    Ready,
    Playing,
    Loss,
    Win,
};

struct Pointer {
    size_t x;
    size_t y;
};

struct _MinesweeperBoard {
    struct Board board;
    struct Board game_board;
    struct Pointer *flagedList;
    size_t left;
    size_t right;
    size_t chording;
    size_t ces;
    size_t flag;
    size_t solved3BV;
    size_t row;
    size_t column;
    enum MouseState mouse_state;
    enum GameBoardState game_board_state;
    size_t pointer_x;
    size_t pointer_y;
};

// class MinesweeperBoard{
// private:
//     struct _MinesweeperBoard core;
// public:
//     MinesweeperBoard(struct Board board);
//     // void callback(int32_t a){
//     //     rust_callback(&(this->data), a);
//     //     // this->info = (uint32_t)a;
//     // };
//     size_t get_row();
// };

// MinesweeperBoard::MinesweeperBoard(Board board);


extern "C" {
    void free_board(struct Board b);
    void free_board_poss(struct BoardPossReturn b);
    size_t cal3BV(struct Board board);
    struct Board laymine(size_t row, size_t column, size_t MineNum, size_t X0, size_t Y0);
    struct BoardPossReturn cal_possibility_onboard(struct Board board_of_game, double mine_num);
    // struct _MinesweeperBoard minesweeperboard_new(struct Board board);
}

// 编译命令
// cl /EHsc test.cpp
// ./test








