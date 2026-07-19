import ms_toollib as ms
import time

board = [
    [1, 1, 2, 1, 1, 0, 0, 0],
    [1, -1, 2, -1, 1, 0, 0, 0],
    [1, 1, 2, 1, 1, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [2, 2, 1, 0, 0, 0, 0, 0],
    [-1, -1, 2, 0, 0, 1, 1, 1],
    [-1, -1, 3, 0, 0, 2, -1, 2],
    [-1, -1, 2, 0, 0, 2, -1, 2],
]

video = ms.BaseVideo(board, 16)
time.sleep(0.6)

video.step("rc", (17, 16))
video.step("rr", (17, 16))
video.step("rc", (16, 49))
time.sleep(0.02)
video.step("rr", (16, 50))
video.step("mv", (48, 51))
video.step("mv", (42, 48))
time.sleep(0.02)
video.step("lc", (16, 32))
time.sleep(0.02)
video.step("lr", (16, 32))
time.sleep(0.02)
video.step("lc", (52, 0))
video.step("lr", (53, 0))
video.step("lc", (16, 32))
video.step("rc", (16, 32))
time.sleep(0.05)
video.step("rr", (16, 32))
time.sleep(0.05)
video.step("lr", (16, 32))
time.sleep(0.05)
video.step("lc", (0, 16))
time.sleep(0.05)
video.step("rc", (0, 16))
time.sleep(0.05)
video.step("rr", (0, 16))
print(f"left_s: {video.left_s}")
time.sleep(0.05)
video.step("lr", (0, 16))
video.step("mv", (112, 112))
video.step("lc", (112, 112))
video.step("lr", (112, 112))
video.step("lc", (97, 112))
video.step("lr", (97, 112))

video.player_identifier = "eee555"
video.race_identifier = "G8888"
video.software = "a test software"
video.country = "CN"

print(f"game_board: {video.game_board}")
print(f"player_identifier: {video.player_identifier}")
print(f"game_board_state: {video.game_board_state}")
print(f"bbbv_solved/bbbv: {video.bbbv_solved}/{video.bbbv}")
print(f"ce: {video.ce}")
print(f"row: {video.row}")
print(f"mine_num: {video.mine_num}")
print(f"rtime: {video.rtime}")
print(f"rtime_ms: {video.rtime_ms}")
print(f"is win: {video.is_completed}")
print(f"STNB: {video.stnb}")
print(f"start_time: {video.start_time}")
print(f"end_time: {video.end_time}")
print(f"path: {video.path}")
print(f"etime: {video.etime}")
print(f"op: {video.op}")
print(f"cell0: {video.cell0}")
print(f"pluck: {video.pluck}")
