CFLAGS+=-O2

all: ../target/release/echo-server

../target/release/echo-server: src/main.rs
	cargo build --release

install: ../target/release/echo-server
	cp ../../target/release/echo-server /Library/PrivilegedHelperTools/com.example.echo
	cp com.example.echo.plist /Library/LaunchDaemons/
	launchctl load /Library/LaunchDaemons/com.example.echo.plist

uninstall:
	launchctl unload /Library/LaunchDaemons/com.example.echo.plist
	rm /Library/PrivilegedHelperTools/com.example.echo
	rm /Library/LaunchDaemons/com.example.echo.plist
