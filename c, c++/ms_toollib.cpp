#include <stdio.h>
#include <stdint.h>
# include "ms_toollib.h"
#include <iostream>

using namespace std;

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
// cl /EHsc caller.cpp
// ./ms_toollib


int main(void) {

    int32_t test_beg_board[] = {1, 1, 1, 1, 1, 2, 2, 2, 1, -1, 1, 2, -1, 3, -1, -1, 1, 1, 1, 3, -1, 5, 3, 3, 0, 0, 0, 2, -1, 3, -1, 1, 0, 0, 0, 1, 1, 3, 2, 2, 0, 0, 0, 0, 0, 2, -1, 2, 0, 1, 1, 1, 0, 2, -1, 2, 0, 1, -1, 1, 0, 1, 1, 1};
    int32_t *n_board = test_beg_board;
    size_t bbbv = cal3BV(n_board, 8, 8);
    printf("The test big board 3bv = %zu;\n", bbbv);


    struct Board board_exp = laymine(16, 30, 99, 0, 0);
    cout << "Print the exp board:\n";
    for(int i = 0; i < board_exp.n_row; i++) {
        for(int j = 0; j < board_exp.n_column; j++) {
            printf("%d, ", board_exp.board[i * board_exp.n_column + j]);
        }
        printf("\n");
    }
    free_board(board_exp); // must dealloc





    int32_t test_beg_game_board[] = {0, 0, 1, 10, 10, 10, 10, 10, 0, 0, 2, 10, 10, 10, 10, 10, 1, 1, 3, 11, 10, 10, 10, 10, 10, 10, 4, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10};
    int32_t *n_game_board = test_beg_game_board;
    struct BoardPossReturn board_poss_return = cal_possibility_onboard(n_game_board, 8, 8, 10);
    cout << "\nPrint the beg game board possibility(is mine):\n";
    for(int i = 0; i < 8; i++) {
        for(int j = 0; j < 8; j++) {
            printf("%f, ", board_poss_return.board_poss[i * 8 + j]);
        }
        printf("\n");
    }
    free_board_poss(board_poss_return); // must dealloc




}






