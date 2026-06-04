#ifndef MS_TOOLLIB_PROBABILITY_H
#define MS_TOOLLIB_PROBABILITY_H

#include <stddef.h>
#include "board.h"

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

struct BoardPossReturn cal_probability_onboard(struct Board board_of_game, double mine_num);
void free_board_poss(struct BoardPossReturn b);

#endif
