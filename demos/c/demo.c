#include <stdio.h>
#include <stdint.h>
#include <malloc.h>
#include "ms_toollib/board.h"
#include "ms_toollib/probability.h"
#include "ms_toollib/zini.h"

static struct Board make_board(size_t row, size_t col, int32_t src[][8]) {
    struct Board b = {
        (struct Row *)malloc(row * sizeof(struct Row)), row
    };
    for (size_t i = 0; i < row; i++) {
        b.rows[i] = (struct Row){ (int32_t *)malloc(col * sizeof(int32_t)), col };
        for (size_t j = 0; j < col; j++)
            b.rows[i].cells[j] = src[i][j];
    }
    return b;
}

static void free_test_board(struct Board b) {
    for (size_t i = 0; i < b.n_row; i++) free(b.rows[i].cells);
    free(b.rows);
}

int main(void) {
    printf("=== ms_toollib C Demo ===\n\n");

    /* ── laymine ── */
    printf("-- laymine (16x30, 99 mines, safe at 0,0) --\n");
    struct Board exp = { NULL, 0 };
    exp = laymine(16, 30, 99, 0, 0);
    printf("rows=%zu cols=%zu\n", exp.n_row, exp.rows->n_column);
    free_board(exp);

    /* ── 3BV · ZiNi · Isl · Op ── */
    int32_t raw[8][8] = {
        {1, 1, 1, 1, 1, 2, 2, 2},
        {1,-1, 1, 2,-1, 3,-1,-1},
        {1, 1, 1, 3,-1, 5, 3, 3},
        {0, 0, 0, 2,-1, 3,-1, 1},
        {0, 0, 0, 1, 1, 3, 2, 2},
        {0, 0, 0, 0, 0, 2,-1, 2},
        {0, 1, 1, 1, 0, 2,-1, 2},
        {0, 1,-1, 1, 0, 1, 1, 1},
    };
    struct Board b = make_board(8, 8, raw);
    printf("\n-- Board Analysis (8x8) --\n");
    printf(" 3BV:  %zu\n", cal_bbbv(b));
    printf(" ZiNi: %zu\n", cal_zini(b));
    printf("HZiNi: %zu\n", cal_hzini(b));
    printf("RZiNi: %zu (50 iters)\n", cal_rzini(b, 50));
    printf(" Isl:  %zu\n", cal_isl(b));
    printf(" Op:   %zu\n", cal_op(b));
    free_test_board(b);

    /* ── Probability ── */
    int32_t game[8][8] = {
        {0, 0, 1,10,10,10,10,10},
        {0, 0, 2,10,10,10,10,10},
        {1, 1, 3,11,10,10,10,10},
        {10,10,4,10,10,10,10,10},
        {10,10,10,10,10,10,10,10},
        {10,10,10,10,10,10,10,10},
        {10,10,10,10,10,10,10,10},
        {10,10,10,10,10,10,10,10},
    };
    struct Board g = make_board(8, 8, game);
    struct BoardPossReturn pr = cal_probability_onboard(g, 10.0);
    printf("\n-- Probability (mine density 10.0) --\n");
    printf(" mines: %zu-%zu-%zu\n", pr.min_mine_num, pr.mine_num, pr.max_mine_num);
    for (size_t i = 0; i < g.n_row; i++) {
        for (size_t j = 0; j < g.rows->n_column; j++)
            printf("%.4f ", pr.board_poss.rows_poss[i].cells_poss[j]);
        putchar('\n');
    }
    free_board_poss(pr);
    free_test_board(g);

    printf("\n=== Done ===\n");
    return 0;
}
