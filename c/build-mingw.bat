:: MSVC直接cargo build --release即可，MinGW需下载x86_64-pc-windows-gnu
:: 换清华源大幅提升下载速度，如已设置便无需
set RUSTUP_DIST_SERVER=https://mirrors.tuna.tsinghua.edu.cn/rustup
rustup target add x86_64-pc-windows-gnu

:: 如已下载x86_64-pc-windows-gnu，直接构建
cargo build --release --target x86_64-pc-windows-gnu
