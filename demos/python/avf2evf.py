import ms_toollib as ms
file_path = r"../../test_files/HI-SCORE Exp_49.25_3BV=127_3BVs=2.57_Wang Jianing G01825.avf"
v = ms.AvfVideo(file_path)
v.parse_video()
v.analyse()
# 生成v4版本的evf文件数据
v.generate_evf_v4_raw_data()
# 创建文件。如重名，自动添加(2)、(3)以此类推
v.save_to_evf_file("evf_file_from_avf")