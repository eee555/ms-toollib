import ms_toollib as ms

evfs = ms.Evfs("../../test_files/[lag]二问题无法_20251113_105617_17.evfs")
evfs.parse()
evfs.analyse()

assert evfs.len() == 17

cell2 = evfs[1]
video2 = cell2.evf_video

game_board_state = video2.game_board_state
video_start_time = video2.video_start_time

for idcell, cell in enumerate(evfs[1:]):
    print(cell.evf_video.start_time, " -> ", cell.evf_video.end_time)

