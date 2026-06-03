import ms_toollib as ms

file_path = r"../../test_files/temp.mvf"

v = ms.MvfVideo(file_path) # 第一步，读取文件的二进制内容
v.parse() # 第二步，解析文件的二进制内容
v.analyse() # 第三步，根据解析到的内容，推衍整个局面
v.current_time = 999.999 # set time to the end of the v
print(v.start_time)
print(v.end_time)

