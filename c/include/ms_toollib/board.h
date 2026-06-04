#ifndef MS_TOOLLIB_BOARD_H
#define MS_TOOLLIB_BOARD_H

#include <stddef.h>
#include <stdint.h>

struct Row {
    int32_t *cells;
    size_t n_column;
};

struct Board {
    struct Row *rows;
    size_t n_row;
};

size_t cal_bbbv(struct Board board);
size_t cal_isl(struct Board board);
size_t cal_op(struct Board board);
struct Board laymine(size_t row, size_t column, size_t MineNum, size_t X0, size_t Y0);
void free_board(struct Board b);

#endif
