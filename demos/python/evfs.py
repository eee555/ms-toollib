import ms_toollib as ms

# 构建evf和evfs演示
# 先后模拟扫雷产生3个evf对象，然后塞入evfs

evfs = ms.Evfs()

# 开始第一局
pixsize = 16
board_1 = ms.laymine(16, 30, 99, 0, 0)
game_1 = ms.BaseVideo(board_1, pixsize)
# 随便点，保证失败。因为随机局面无法保证成功。
for i in range(16):
    for j in range(30):
        game_1.step("lc", (i * pixsize, j * pixsize))
        game_1.step("mv", (100, 100))
        game_1.step("lr", (i * pixsize, j * pixsize))

assert game_1.game_board_state == 4 # fail
game_1.player_identifier = "name1"
game_1.generate_evf_v4_raw_data()
game_1.checksum_evf_v4 = [4, 7, 3, 5]
filename = "./test_1"
game_1.save_to_evf_file(filename)
checksum_cell_1 = [i for i in range(32)]
evfs.append(game_1.raw_data, filename, checksum_cell_1)



# 开始第二局
pixsize = 20
board_2 = ms.laymine(8, 8, 10, 5, 3)
game_2 = ms.BaseVideo(board_2, pixsize)
# 随便点，保证失败。因为随机局面无法保证成功。
for i in range(8):
    for j in range(8):
        game_2.step("lc", (i * pixsize, j * pixsize))
        game_2.step("lr", (i * pixsize, j * pixsize))

assert game_2.game_board_state == 4 # fail
game_2.player_identifier = "name2"
game_2.generate_evf_v4_raw_data()
game_2.checksum_evf_v4 = [2, 3, 4, 7, 4, 4, 43]
filename = "./test_2"
game_2.save_to_evf_file(filename)
checksum_cell_2 = [i + 1 for i in range(32)]
evfs.append(game_2.raw_data, filename, checksum_cell_2)



# 开始第三局
pixsize = 16
board_3 = ms.laymine(16, 30, 99, 0, 0)
game_3 = ms.BaseVideo(board_3, pixsize)
# 随便点，保证失败。因为随机局面无法保证成功。
for i in range(16):
    for j in range(30):
        game_3.step("lc", (i * pixsize, j * pixsize))
        game_3.step("mv", (100, 100))
        game_3.step("lr", (i * pixsize, j * pixsize))
assert game_3.game_board_state == 4 # fail
game_3.player_identifier = "name3"
game_3.generate_evf_v4_raw_data()
game_3.checksum_evf_v4 = [1, 2, 3]
filename = "./test_3"
game_3.save_to_evf_file(filename)
checksum_cell_3 = [i + 1 for i in range(32)]
evfs.append(game_3.raw_data, filename, checksum_cell_3)


# 扫完3局evf，一起保存成evfs
evfs_filename = "./test"
evfs.generate_evfs_v0_raw_data()
evfs.save_evfs_file(evfs_filename)

# 重新读取刚刚保存的evfs，验证正确性
evfs_reload: ms.Evfs = ms.Evfs(file_name=evfs_filename+".evfs")
evfs_reload.parse()
assert evfs_reload.len() == 3
video_1: ms.EvfVideo = evfs_reload[0].evf_video
assert video_1.player_identifier == "name1"
video_1.analyse()
for e in video_1.events:
    print(e.time, e.x, e.y, e.mouse, e.path, e.comments, e.mouse_state)


