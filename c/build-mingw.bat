:: MSVC直接cargo build --release即可，MinGW需下载x86_64-pc-windows-gnu
:: 换清华源大幅提升下载速度，如已设置便无需
set RUSTUP_DIST_SERVER=https://mirrors.tuna.tsinghua.edu.cn/rustup
rustup target add x86_64-pc-windows-gnu

:: 如已下载x86_64-pc-windows-gnu，直接构建
cargo build --release --target x86_64-pc-windows-gnu

:: 原静态库编译程序会产生3.6万条警告，因库中包含MSVC风格的链接器指令（.drectve段）
:: 可利用MinGW的strip指令清除，需要添加MinGW的bin文件夹到环境变量
::cd target\x86_64-pc-windows-gnu\release
::strip --strip-unneeded libms_toollib.a  :: 新版已不再需要
