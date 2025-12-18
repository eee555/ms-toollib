#include "cxxbridge_code/src/lib.rs.h"
#include <iostream>
#include <string>
#include "rust/cxx.h"

// cmake -B build . && make -C build -j4
// build/cxx_cmake

// struct Vectori32 {
//     std::vector<int32_t> vector;
// };

// CxxVeci32::CxxVeci32(char *name, int age, float score){
//     vec = 2;
// }

int main()
{
    rust::cxxbridge1::Box<AvfVideo> v = new_AvfVideo("jze.avf");
    v->parse();
    v->analyse();
    rust::string player_name = v->get_player();

    std::cout << "bbbv is: " << v->get_bbbv() << std::endl;
    std::cout << "row is: " << v->get_row() << std::endl;
    std::cout << "win is: " << v->get_win() << std::endl;
    std::cout << "player is: " << player_name << std::endl;

    std::cout << std::endl
              << "laymine result is: " << std::endl;
    auto laymine_board = laymine(6, 9, 10, 0, 5);
    for (int i = 0; i < laymine_board.size(); i++)
    {
        for (int j = 0; j < laymine_board[0].vec.size(); j++)
        {
            std::cout << laymine_board[i].vec[j] << ", ";
        }
        std::cout << std::endl;
    }
    std::cout << std::endl;

    std::cout << "laymine_solvable_thread result is: " << std::endl;
    auto laymine_solvable_thread_board_flag = laymine_solvable_thread(8, 8, 10, 7, 2, 100000);
    auto laymine_solvable_thread_board = laymine_solvable_thread_board_flag.board;
    for (int i = 0; i < laymine_solvable_thread_board.size(); i++)
    {
        for (int j = 0; j < laymine_solvable_thread_board[0].vec.size(); j++)
        {
            std::cout << laymine_solvable_thread_board[i].vec[j] << ", ";
        }
        std::cout << std::endl;
    }
    std::cout << std::endl;

    int test_beg_game_board[8][8] = {{0, 0, 1, 10, 10, 10, 10, 2},
                                     {0, 0, 2, 10, 10, 10, 10, 10},
                                     {1, 1, 3, 11, 10, 10, 10, 10},
                                     {10, 10, 4, 10, 10, 10, 10, 10},
                                     {10, 10, 10, 10, 10, 10, 10, 10},
                                     {10, 10, 10, 10, 10, 10, 10, 10},
                                     {10, 10, 10, 10, 10, 10, 10, 10},
                                     {10, 10, 10, 10, 10, 10, 10, 10}};
    // std::vector<int> temp = {};
    std::vector<int32_t> beg_game_board_1 = {};
    for (int i = 0; i < 8; i++)
    {
        // std::vector<int> temp = {};
        for (int j = 0; j < 8; j++)
        {
            beg_game_board_1.push_back(test_beg_game_board[i][j]);
        }
    }
    BoardPossReturn poss_a = cal_probability_onboard(beg_game_board_1, 8, 10.0);

    for (int i = 0; i < poss_a.board_poss.size(); i++)
    {
        for (int j = 0; j < poss_a.board_poss[0].vec.size(); j++)
        {
            std::cout << poss_a.board_poss[i].vec[j] << ", ";
        }
        std::cout << std::endl;
    }
    std::cout << std::endl;

    return 0;
}
