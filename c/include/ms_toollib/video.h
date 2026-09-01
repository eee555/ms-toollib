#ifndef MS_TOOLLIB_VIDEO_H
#define MS_TOOLLIB_VIDEO_H

#include <stddef.h>
#include <stdint.h>
#include "board.h"      // 提供 Board 结构体
#include "probability.h" // 提供 BoardPossReturn（虽然本文件未直接用，但依赖已含）

#ifdef __cplusplus
extern "C" {  // 对于C++，使用C名称修饰
#endif
	
	/* ---------- AvfVideo ---------- */
	void* avf_video_new(const char* filename);
	void* avf_video_new_from_data(const uint8_t* data, size_t len, const char* filename);
	void  avf_video_free(void* ptr);
	int32_t avf_video_parse(void* ptr);
	void* avf_video_data_ptr(void* ptr);
	
	/* ---------- EvfVideo ---------- */
	void* evf_video_new(const char* filename);
	void* evf_video_new_from_data(const uint8_t* data, size_t len, const char* filename);
	void  evf_video_free(void* ptr);
	int32_t evf_video_parse(void* ptr);
	void* evf_video_data_ptr(void* ptr);
	
	/* ---------- MvfVideo ---------- */
	void* mvf_video_new(const char* filename);
	void* mvf_video_new_from_data(const uint8_t* data, size_t len, const char* filename);
	void  mvf_video_free(void* ptr);
	int32_t mvf_video_parse(void* ptr);
	void* mvf_video_data_ptr(void* ptr);
	
	/* ---------- RmvVideo ---------- */
	void* rmv_video_new(const char* filename);
	void* rmv_video_new_from_data(const uint8_t* data, size_t len, const char* filename);
	void  rmv_video_free(void* ptr);
	int32_t rmv_video_parse(void* ptr);
	void* rmv_video_data_ptr(void* ptr);
	
	/* ---------- BaseVideo common operations ---------- */
	void    base_video_analyse(void* ptr);
	double  base_video_get_rtime(void* ptr);
	uint32_t base_video_get_rtime_ms(void* ptr);
	double  base_video_get_etime(void* ptr);
	struct Board base_video_get_game_board(void* ptr);       // 返回的 Board 需用 free_board() 释放
	size_t  base_video_get_left(void* ptr);
	size_t  base_video_get_right(void* ptr);
	size_t  base_video_get_double(void* ptr);
	size_t  base_video_get_cl(void* ptr);
	size_t  base_video_get_flag(void* ptr);
	size_t  base_video_get_bbbv_solved(void* ptr);
	size_t  base_video_get_ce(void* ptr);
	double  base_video_get_corr(void* ptr);
	double  base_video_get_thrp(void* ptr);
	double  base_video_get_ioe(void* ptr);
	double  base_video_get_path(void* ptr);
	double  base_video_get_stnb(void* ptr);
	size_t  base_video_get_mouse_state(void* ptr);
	size_t  base_video_get_current_event_id(void* ptr);
	uint8_t base_video_set_current_event_id(void* ptr, size_t id);
	double  base_video_get_current_time(void* ptr);
	void    base_video_set_current_time(void* ptr, double t);
	size_t  base_video_get_event_count(void* ptr);
	uint8_t base_video_is_valid(void* ptr);
	
	/* ---------- Event accessors ---------- */
	double  base_video_event_time(void* ptr, size_t idx);
	char*   base_video_event_desc(void* ptr, size_t idx);   // 返回的字符串需用 base_video_free_event_desc 释放
	void    base_video_free_event_desc(char* s);
	
	/* ---------- Metadata getters ---------- */
	char*   base_video_get_software(void* ptr);    // 需用 base_video_free_string 释放
	char*   base_video_get_player(void* ptr);      // 需用 base_video_free_string 释放
	void    base_video_free_string(char* s);
	
	size_t   base_video_get_width(void* ptr);
	size_t   base_video_get_height(void* ptr);
	size_t   base_video_get_mine_num(void* ptr);
	uint16_t base_video_get_mode(void* ptr);
	uint8_t  base_video_get_level(void* ptr);
	uint8_t  base_video_get_nf(void* ptr);
	uint8_t  base_video_get_is_completed(void* ptr);
	
#ifdef __cplusplus
}
#endif

#endif /* MS_TOOLLIB_VIDEO_H */
