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

/* ---------- 辅助：读取文件名 ---------- */
char* InputFileName(void) {
	char temp[256];
	int i, j;
	char* fileName = (char*)calloc(256, sizeof(char));
	printf("[File]>");
	fgets(temp, 256, stdin);
	j = 0;
	for (i = 0; i < 256; i++) {
		if (temp[i] == '\n') break;
		else if (temp[i] != '"') { // 过滤双引号
			fileName[j] = temp[i];
			j++;
		}
	}
	return fileName;
}

/* ---------- 判断后缀 ---------- */
static int ends_with(const char *str, const char *suffix) {
	size_t len_str = strlen(str);
	size_t len_suf = strlen(suffix);
	if (len_suf > len_str) return 0;
	return strcmp(str + len_str - len_suf, suffix) == 0;
}

/* ---------- 打印 Board（辅助） ---------- */
static void print_board(const char *title, struct Board b) {
	printf("\n%s:\n", title);
	for (size_t i = 0; i < b.n_row; i++) {
		for (size_t j = 0; j < b.rows->n_column; j++) {
			printf("%4d ", b.rows[i].cells[j]);
		}
		printf("\n");
	}
}

/* ---------- 主测试函数 ---------- */
int main(void) {
	char *filename = InputFileName();
	if (!filename || strlen(filename) == 0) {
		printf("Invalid filename.\n");
		free(filename);
		return 1;
	}
	
	void *video = NULL;
	void *data = NULL;
	int (*parse_func)(void*) = NULL;
	void (*free_func)(void*) = NULL;
	void* (*data_ptr_func)(void*) = NULL;
	
	// 根据后缀选择对应的函数族
	if (ends_with(filename, ".avf")) {
		printf("Detected .avf format.\n");
		video = avf_video_new(filename);
		parse_func = avf_video_parse;
		free_func = avf_video_free;
		data_ptr_func = avf_video_data_ptr;
	} else if (ends_with(filename, ".evf")) {
		printf("Detected .evf format.\n");
		video = evf_video_new(filename);
		parse_func = evf_video_parse;
		free_func = evf_video_free;
		data_ptr_func = evf_video_data_ptr;
	} else if (ends_with(filename, ".mvf")) {
		printf("Detected .mvf format.\n");
		video = mvf_video_new(filename);
		parse_func = mvf_video_parse;
		free_func = mvf_video_free;
		data_ptr_func = mvf_video_data_ptr;
	} else if (ends_with(filename, ".rmv")) {
		printf("Detected .rmv format.\n");
		video = rmv_video_new(filename);
		parse_func = rmv_video_parse;
		free_func = rmv_video_free;
		data_ptr_func = rmv_video_data_ptr;
	} else {
		printf("Unsupported file extension. Only .avf, .evf, .mvf, .rmv are supported.\n");
		free(filename);
		return 1;
	}
	
	free(filename); // 不再需要文件名
	
	if (!video) {
		printf("Failed to create video object.\n");
		return 1;
	}
	
	// 解析视频
	if (parse_func(video) != 0) {
		printf("Video parsing failed.\n");
		free_func(video);
		return 1;
	}
	
	// 获取 BaseVideo 数据指针（所有 base_video_* 函数都使用这个指针）
	data = data_ptr_func(video);
	if (!data) {
		printf("Failed to get data pointer.\n");
		free_func(video);
		return 1;
	}
	
	// ---------- 打印基本信息 ----------
	printf("\n========== Video Metadata ==========\n");
	char *software = base_video_get_software(data);
	char *player   = base_video_get_player(data);
	printf("Software: %s\n", software ? software : "(null)");
	printf("Player:   %s\n", player   ? player   : "(null)");
	base_video_free_string(software);
	base_video_free_string(player);
	
	printf("Width:      %zu\n", base_video_get_width(data));
	printf("Height:     %zu\n", base_video_get_height(data));
	printf("Mine num:   %zu\n", base_video_get_mine_num(data));
	printf("Mode:       %u\n",  base_video_get_mode(data));
	printf("Level:      %u\n",  base_video_get_level(data));
	printf("NF:         %u\n",  base_video_get_nf(data));
	printf("Completed:  %u\n",  base_video_get_is_completed(data));
	printf("Valid:      %u\n",  base_video_is_valid(data));
	
	// ---------- 时间信息 ----------
	printf("\n========== Timing ==========\n");
	printf("RTime (s):    %f\n", base_video_get_rtime(data));
	printf("RTime (ms):   %u\n",  base_video_get_rtime_ms(data));
	printf("ETime (s):    %f\n", base_video_get_etime(data));
	printf("Current time: %f\n", base_video_get_current_time(data));
	
	// ---------- 动作计数 ----------
	printf("\n========== Action Counts ==========\n");
	printf("Left:   %zu\n", base_video_get_left(data));
	printf("Right:  %zu\n", base_video_get_right(data));
	printf("Double: %zu\n", base_video_get_double(data));
	printf("CL:     %zu\n", base_video_get_cl(data));
	printf("Flag:   %zu\n", base_video_get_flag(data));
	
	// ---------- 求解指标 ----------
	printf("\n========== Solving Metrics ==========\n");
	printf("3BV solved: %zu\n", base_video_get_bbbv_solved(data));
	printf("CE:         %zu\n", base_video_get_ce(data));
	printf("Corr:       %f\n", base_video_get_corr(data));
	printf("Thrp:       %f\n", base_video_get_thrp(data));
	printf("IOE:        %f\n", base_video_get_ioe(data));
	printf("Path:       %f\n", base_video_get_path(data));
	printf("Stnb:       %f\n", base_video_get_stnb(data));
	
	// ---------- 鼠标状态 ----------
	printf("\nMouse state (0=UpUp,1=UpDown,...): %zu\n", base_video_get_mouse_state(data));
	
	// ---------- 事件系统 ----------
	size_t event_count = base_video_get_event_count(data);
	printf("\nEvent count: %zu\n", event_count);
	
	size_t current_id = base_video_get_current_event_id(data);
	printf("Current event ID: %zu\n", current_id);
	// 尝试设置当前事件为0（测试）
	uint8_t set_ok = base_video_set_current_event_id(data, 0);
	printf("Set current event to 0: %s\n", set_ok ? "OK" : "Failed");
	// 恢复原ID（方便后续遍历）
	base_video_set_current_event_id(data, current_id);
	
	printf("\n--- Event List (up to first 20) ---\n");
	size_t max_print = event_count < 20 ? event_count : 20;
	for (size_t i = 0; i < max_print; i++) {
		double t = base_video_event_time(data, i);
		char *desc = base_video_event_desc(data, i);
		printf("[%zu] time=%.3f  desc=%s\n", i, t, desc ? desc : "(null)");
		base_video_free_event_desc(desc);
	}
	if (event_count > 20) {
		printf("... (truncated)\n");
	}
	
	// ---------- 当前游戏盘面 ----------
	struct Board game_board = base_video_get_game_board(data);
	print_board("Current Game Board", game_board);
	free_board(game_board);   // 必须释放
	
	// ---------- 调用 analyse（无输出，仅触发内部计算） ----------
	base_video_analyse(data);
	printf("\nAnalyse called (no output).\n");
	
	// ---------- 测试 set_current_time ----------
	double old_time = base_video_get_current_time(data);
	base_video_set_current_time(data, 123.456);
	printf("Current time after set: %f (restored later)\n", base_video_get_current_time(data));
	base_video_set_current_time(data, old_time);
	
	// ---------- 释放 ----------
	free_func(video);   // 释放整个视频对象（包含 data 内部）
	printf("\nAll resources released. Test finished.\n");
	getchar();
	return 0;
}
