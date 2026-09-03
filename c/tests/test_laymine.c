#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include "ms_toollib/ms_toollib.h"

#ifdef _MSC_VER
	#pragma comment(lib, "ws2_32.lib")
	#pragma comment(lib, "Advapi32.lib")
	#pragma comment(lib, "Iphlpapi.lib")
	#pragma comment(lib, "Psapi.lib")
	#pragma comment(lib, "user32.lib")
	#pragma comment(lib, "userenv.lib")
	#pragma comment(lib, "bcrypt.lib")
	#pragma comment(lib, "ntdll.lib")
	#pragma comment(lib, "target/release/ms_toollib.lib")
#endif

/* ---------- 打印盘面，并标注起始位置 ---------- */
static void print_board_with_start(const char *title, struct Board b, size_t x0, size_t y0, int success) {
	printf("\n%s (success=%d)\n", title, success);
	if(b.n_row == 0 || b.rows == NULL) {
		printf("(empty board)\n");
		return;
	}
	// 先确定列数（假设所有行等宽）
	size_t cols = b.rows->n_column;
	
	// 打印列号（可选）
	printf("   ");
	for(size_t j = 0; j < cols; j++) {
		printf("%2zu ", j);
	}
	printf("\n");
	
	for(size_t i = 0; i < b.n_row; i++) {
		// 打印行号
		printf("%2zu ", i);
		for(size_t j = 0; j < cols; j++) {
			int32_t val = b.rows[i].cells[j];
			// 标注起始位置
			if(i == x0 && j == y0) {
				// 如果该处是雷（-1）则显示 * 加括号，否则显示数字加括号
				if(val == -1) {
					printf("(* )");
				} else {
					printf("(%d)", val);
				}
			} else {
				// 普通显示：雷显示 'X'，数字显示数字
				if(val == -1) {
					printf(" X ");
				} else {
					printf("%2d ", val);
				}
			}
		}
		printf("\n");
	}
}

/* ---------- 主函数 ---------- */
int main(void) {
	// 测试参数：高级棋盘大小 16x30，雷数 99，起手位置 (3, 20)
	const size_t ROW = 16;
	const size_t COL = 30;
	const size_t MINE = 99;
	const size_t X0 = 3;
	const size_t Y0 = 20;
	
	printf("========== Test laymine_op ==========\n");
	struct Board board_op = laymine_op(ROW, COL, MINE, X0, Y0);
	print_board_with_start("laymine_op (win7 rule)", board_op, X0, Y0, 1);
	free_board(board_op);
	
	// 测试 is_solvable 对 laymine_op 生成的局面（可能无猜，可能不）
	printf("\n========== Test is_solvable on laymine_op ==========\n");
	struct Board board_sol_test = laymine_op(ROW, COL, MINE, X0, Y0);
	uint8_t solvable = is_solvable(board_sol_test, X0, Y0);
	printf("is_solvable on laymine_op board: %s\n", solvable ? "YES" : "NO");
	free_board(board_sol_test);
	
	// 测试 laymine_solvable（筛选法，尝试次数设大一些）
	printf("\n========== Test laymine_solvable ==========\n");
	size_t max_tries = 100000;
	struct BoardReturn ret1 = laymine_solvable(ROW, COL, MINE, X0, Y0, max_tries);
	print_board_with_start("laymine_solvable (filtering)", ret1.board, X0, Y0, ret1.success);
	free_board(ret1.board);
	
	// 测试 laymine_solvable_adjust（调整法）
	printf("\n========== Test laymine_solvable_adjust ==========\n");
	struct BoardReturn ret2 = laymine_solvable_adjust(ROW, COL, MINE + 50, X0, Y0); // 更高雷数测试
	print_board_with_start("laymine_solvable_adjust (adjustment)", ret2.board, X0, Y0, ret2.success);
	free_board(ret2.board);
	
	// 再测试一个较小的棋盘，方便观察
	printf("\n========== Small board test (8x8, 10 mines) ==========\n");
	const size_t ROW2 = 8, COL2 = 8, MINE2 = 10, X02 = 2, Y02 = 3;
	struct BoardReturn ret3 = laymine_solvable_adjust(ROW2, COL2, MINE2, X02, Y02);
	print_board_with_start("Small board adjust", ret3.board, X02, Y02, ret3.success);
	free_board(ret3.board);
	
	printf("\nAll tests done. Press Enter to exit.\n");
	getchar();
	return 0;
}
