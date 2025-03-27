// #include <cstdint>

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

void free_board(struct Board b);
void free_board_poss(struct BoardPossReturn b);
size_t cal3BV(struct Board board);
struct Board laymine(size_t row, size_t column, size_t MineNum, size_t X0, size_t Y0);
struct BoardPossReturn cal_probability_onboard(struct Board board_of_game, double mine_num);

// 编译命令
// cl /EHsc test.c
// ./test








