CARGO := cargo
CC := cl
CFLAGS := /EHsc
RUST_LIB := target/release/ms_toollib.lib
WIN_LIBS := ws2_32.lib Advapi32.lib Iphlpapi.lib Psapi.lib user32.lib userenv.lib bcrypt.lib ntdll.lib

.PHONY: all build test clean

all: build

build:
	$(CARGO) build --release

tests/test_board.exe: tests/test_board.c build
	$(CC) $(CFLAGS) /Iinclude tests/test_board.c /link $(RUST_LIB) $(WIN_LIBS)

test: tests/test_board.exe
	tests/test_board.exe

clean:
	cargo clean
	-del /f tests\test_board.exe tests\test_board.obj 2>nul || ver>nul
