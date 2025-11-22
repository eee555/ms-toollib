import ms_toollib as ms

evfs = ms.Evfs("../../test_files/[lag]二问题无法_20251113_105617_17.evfs")
evfs.parse()
evfs.analyse()

cell2 = evfs[1]
video2 = cell2.evf_video

game_board_state = video2.game_board_state
video_start_time = video2.video_start_time
