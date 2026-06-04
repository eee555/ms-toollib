JAVA_HOME := D:/Android/Android\ Studio/jbr
JAVAC     := "$(JAVA_HOME)/bin/javac"
JAVA      := "$(JAVA_HOME)/bin/java"

.PHONY: all build run clean

all: build run

build: Demo.class

Demo.class: Demo.java
	$(JAVAC) -cp "../../java/target/classes" Demo.java

run:
	$(JAVA) -Djava.library.path="../../java/native" -cp ".;../../java/target/classes" Demo

clean:
	-del /f Demo.class 2>nul || cd .
