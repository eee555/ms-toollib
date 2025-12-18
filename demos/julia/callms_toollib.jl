# ENV["PYTHON"] = raw"D:\Python\Python310\python.exe"
# using Pkg
# Pkg.add("PyCall")
# Pkg.build("PyCall")
using PyCall
# pip install ms_toollib
@pyimport ms_toollib as ms
v = ms.AvfVideo("../../test_files/HI-SCORE Exp_49.25_3BV=127_3BVs=2.57_Wang Jianing G01825.avf") # 第一步，读取文件的二进制内容
v.parse() # 第二步，解析文件的二进制内容
v.analyse() # 第三步，根据解析到的内容，推衍整个局面
@assert v.player_identifier=="Wang Jianing G01825"
@assert v.bbbv==127
println(ms.laymine(16, 30, 99, 2, 3)) # 埋雷

