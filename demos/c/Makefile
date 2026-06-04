CC      := cl
CFLAGS  := /EHsc /I../../c/include
RUSTLIB := ../../c/target/release/ms_toollib.lib
WINLIBS := ws2_32.lib Advapi32.lib Iphlpapi.lib Psapi.lib user32.lib userenv.lib bcrypt.lib ntdll.lib

.PHONY: all build clean

all: build
	$(MAKE) build
	demo.exe

build:
	@if not exist "$(RUSTLIB)" ( \
		echo Building Rust static library first... && \
		cd ../../c && cargo build --release && cd ../../demos/c \
	)
	$(CC) $(CFLAGS) demo.c /link $(RUSTLIB) $(WINLIBS)

clean:
	-del /f demo.exe demo.obj 2>nul || cd .
