

struct Board {
    int32_t *board;
    size_t n_row;
    size_t n_column;
};

struct BoardPossReturn {
    double *board_poss;
    size_t n_row;
    size_t n_column;
    size_t min_mine_num;
    size_t mine_num;
    size_t max_mine_num;
};


extern "C" {
    void free_board(struct Board b);
    void free_board_poss(struct BoardPossReturn b);
    size_t cal3BV(int32_t* board, size_t n_row, size_t n_column);
    struct Board laymine(size_t row, size_t column, size_t MineNum, size_t X0, size_t Y0);
    struct BoardPossReturn cal_possibility_onboard(int32_t* board_of_game, size_t n_row, size_t n_column, double mine_num);
}

// 编译命令
// cl /EHsc ms_toollib.cpp
// ./ms_toollib








