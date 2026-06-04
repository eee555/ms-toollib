JAVA_HOME := D:/Android/Android\ Studio/jbr
CARGO     := cargo
CC        := cl
CFLAGS    := /EHsc /I"$(JAVA_HOME)/include" /I"$(JAVA_HOME)/include/win32"
CINCLUDE  := /I../c/include
RUSTLIB   := ../c/target/release/ms_toollib.lib
WINLIBS   := ws2_32.lib Advapi32.lib Iphlpapi.lib Psapi.lib user32.lib userenv.lib bcrypt.lib ntdll.lib

JAVAC := "$(JAVA_HOME)/bin/javac"
JAVA  := "$(JAVA_HOME)/bin/java"
JAR   := "$(JAVA_HOME)/bin/jar"

CLASSES := target/classes
JARFILE := target/ms_toollib.jar

.PHONY: all build dll jar clean

all: dll jar

dll:
	@if not exist "$(RUSTLIB)" ( \
		echo Building Rust static library... && \
		cd ../c && $(CARGO) build --release && cd ../java \
	)
	if not exist native mkdir native
	$(CC) $(CFLAGS) $(CINCLUDE) /LD /Fe native/ms_toollib_jni.dll \
		src/main/c/jni_wrapper.c /link $(RUSTLIB) $(WINLIBS)
	-del /f native/ms_toollib_jni.lib native/ms_toollib_jni.exp 2>nul || cd .

jar:
	if not exist $(CLASSES) mkdir $(CLASSES)
	$(JAVAC) -d $(CLASSES) src/main/java/ms_toollib/MsToollib.java
	cd $(CLASSES) && $(JAR) cf $(subst /,\,$(JARFILE)) ms_toollib/

clean:
	-del /f jni_wrapper.obj 2>nul || cd .
	-rmdir /s /q native 2>nul || cd .
	-rmdir /s /q $(CLASSES) 2>nul || cd .
