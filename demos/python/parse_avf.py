import ms_toollib as ms

file_path = r"../../test_files/HI-SCORE Exp_49.25_3BV=127_3BVs=2.57_Wang Jianing G01825.avf"

v = ms.AvfVideo(file_path) # 第一步，读取文件的二进制内容
v.parse_video() # 第二步，解析文件的二进制内容
v.analyse() # 第三步，根据解析到的内容，推衍整个局面
v.current_time = 999.999 # set time to the end of the v
assert v.left == 126
assert v.right == 11
assert v.double == 14
assert v.left_s == 2.5583756345177666
assert v.right_s == 0.2233502538071066
assert v.double_s == 0.28426395939086296
assert v.level == 5
assert v.cl == 151
assert v.cl_s == 3.065989847715736
assert v.ce == 144
assert v.ce_s == 2.9238578680203045
assert v.rce == 11
assert v.lce == 119
assert v.dce == 14
assert v.bbbv == 127
assert v.bbbv_solved == 127
assert v.bbbv_s == 2.5786802030456855
assert v.flag == 11
assert v.path == 6082.352554578606
print(v.time)  # the time shown on the counter currently
print(v.rtime) # game time, shown on leaderboard
print(v.etime) # the estimated time shown on the counter currently
assert v.start_time == 1666124135606000
assert v.end_time == 1666124184868000
assert v.mode == 0
assert v.software == "Arbiter"
assert v.player_identifier == "Wang Jianing G01825"
assert v.race_identifier == ""
assert v.uniqueness_identifier == ""
print(v.stnb)
print(v.corr)
print(v.thrp)
print(v.ioe)
print(v.is_official)
print(v.is_fair)
# 遍历并查看鼠标动作
for e in v.events:
    if e.event.is_mouse():
        e_mouse = e.event.unwrap_mouse()
        print(e.time, e_mouse.x, e_mouse.y, e_mouse.mouse, e.path, e.comments, e.mouse_state)
        # left, right, double, lce, rce, dce, flag, bbbv_solved, op_solved, isl_solved
        print(e.key_dynamic_params.left)
        ...
v.analyse_for_features(["high_risk_guess"]) # 用哪些分析方法。分析结果会记录到events.comments里
for e in v.events:
    if e.useful_level >= 2:
        # 实施鼠标动作前的局面
        builtin_game_board = e.prior_game_board
        # 实施鼠标动作后的局面
        next_game_board = e.next_game_board
        # 内置的游戏局面类的列表类型的游戏局面
        print(builtin_game_board.game_board)
        # 内置的游戏局面类的每格是雷的概率
        print(builtin_game_board.poss)
        # 用1或2个方程求解时，得出的所有非雷的位置
        print(builtin_game_board.basic_not_mine)
        # 用1或2个方程求解时，得出的所有是雷的位置
        print(builtin_game_board.basic_is_mine)
        # 用枚举法求解时，得出的所有非雷的位置，不包括basic_not_mine
        print(builtin_game_board.enum_not_mine)
        # 用枚举法求解时，得出的所有是雷的位置，不包括basic_is_mine
        print(builtin_game_board.enum_is_mine)

# 其他实例化方法
with open(file_path, 'rb') as file:
    video_data_list = file.read() # 或video_data_list = list(file.read())
    # 自定义另一个文件名
    v_2 = ms.AvfVideo(r"my_file.avf", video_data_list)
    # 也可以缺省第一个参数`file_name`，则file_name为空字符串
    v_3 = ms.AvfVideo(raw_data=video_data_list)
