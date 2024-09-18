# ENV["PYTHON"] = raw"D:\Python\Python310\python.exe"
# Pkg.add("PyCall")
# Pkg.build("PyCall")
using PyCall
# pip install ms_toollib
@pyimport ms_toollib as ms
v = ms.AvfVideo("test.avf") # 第一步，读取文件的二进制内容
v.parse_video() # 第二步，解析文件的二进制内容
v.analyse() # 第三步，根据解析到的内容，推衍整个局面
println(decode(Vector{UInt8}(v.player_identifier),"UTF-8")) # 录像的标识
println(v.bbbv) # 录像的3BV
println(ms.laymine(16, 30, 99, 2, 3)) # 埋雷

