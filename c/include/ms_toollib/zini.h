#ifndef MS_TOOLLIB_ZINI_H
#define MS_TOOLLIB_ZINI_H

#include <stddef.h>
#include "board.h"

size_t cal_zini(struct Board board);
size_t cal_hzini(struct Board board);
size_t cal_rzini(struct Board board, size_t n_iter);

#endif
