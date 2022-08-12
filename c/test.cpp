#include <stdio.h>
#include <stdint.h>
# include "ms_toollib.h"
#include<malloc.h>

#pragma comment(lib,"ws2_32.lib")
#pragma comment (lib,"Advapi32.lib")
#pragma comment (lib,"Iphlpapi.lib")
#pragma comment(lib, "Psapi.lib")
#pragma comment(lib, "user32.lib")
#pragma comment(lib, "userenv.lib")
#pragma comment(lib, "bcrypt.lib")

#pragma comment(lib, "./target/release/ms_toollib.lib")

// 编译命令
// cargo build --release
// cl /EHsc test.cpp
// ./test


int main(void) {

    int32_t test_beg_board[8][8] = {{1, 1, 1, 1, 1, 2, 2, 2}, 
    {1, -1, 1, 2, -1, 3, -1, -1}, 
    {1, 1, 1, 3, -1, 5, 3, 3}, 
    {0, 0, 0, 2, -1, 3, -1, 1}, 
    {0, 0, 0, 1, 1, 3, 2, 2}, 
    {0, 0, 0, 0, 0, 2, -1, 2}, 
    {0, 1, 1, 1, 0, 2, -1, 2}, 
    {0, 1, -1, 1, 0, 1, 1, 1}};
    struct Row *rows_ = (struct Row *)malloc(8 * sizeof(struct Row));
    int32_t *cells_;
    struct Board a_test_beg_board = {rows_, 8};
    for(int i = 0; i < 8; i++) {
        cells_ = (int32_t *)malloc(8 * sizeof(int32_t));
        a_test_beg_board.rows[i] = {cells_, 8};
        for(int j = 0; j < 8; j++) {
            a_test_beg_board.rows[i].cells[j] = test_beg_board[i][j];
        };
    };
    size_t bbbv = cal3BV(a_test_beg_board);
    printf("3BV of the beg board is: %zu", bbbv);
    for(int i = 0; i < 8; i++) {
        free((int32_t *)a_test_beg_board.rows[i].cells);
    };
    free(rows_);




    struct Board board_exp = laymine(16, 30, 99, 0, 0);
    printf("\n\nPrint the exp board:\n");
    for(int i = 0; i < board_exp.n_row; i++) {
        for(int j = 0; j < board_exp.rows->n_column; j++) {
            printf("%d, ", board_exp.rows[i].cells[j]);
        }
        printf("\n");
    }
    free_board(board_exp); // must dealloc





    int32_t test_beg_game_board[8][8] = {{0, 0, 1, 10, 10, 10, 10, 10}, 
    {0, 0, 2, 10, 10, 10, 10, 10}, 
    {1, 1, 3, 11, 10, 10, 10, 10}, 
    {10, 10, 4, 10, 10, 10, 10, 10}, 
    {10, 10, 10, 10, 10, 10, 10, 10}, 
    {10, 10, 10, 10, 10, 10, 10, 10}, 
    {10, 10, 10, 10, 10, 10, 10, 10}, 
    {10, 10, 10, 10, 10, 10, 10, 10}};
    struct Row *rows__ = (struct Row *)malloc(8 * sizeof(struct Row));
    int32_t *cells__;
    struct Board a_test_beg_game_board = {rows__, 8};
    for(int i = 0; i < 8; i++) {
        cells__ = (int32_t *)malloc(8 * sizeof(int32_t));
        a_test_beg_game_board.rows[i] = {cells__, 8};
        for(int j = 0; j < 8; j++) {
            a_test_beg_game_board.rows[i].cells[j] = test_beg_game_board[i][j];
        };
    };
    struct BoardPossReturn board_poss_return = cal_possibility_onboard(a_test_beg_game_board, 10.0);
    printf("\nPrint the beg game board possibility(is mine):\n");
    for(int i = 0; i < 8; i++) {
        for(int j = 0; j < 8; j++) {
            printf("%f, ", board_poss_return.board_poss.rows_poss[i].cells_poss[j]);
        }
        printf("\n");
    }
    free_board_poss(board_poss_return); // must dealloc


    MouseState s;


}






