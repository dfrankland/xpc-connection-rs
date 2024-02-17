all: ../../target/debug/echo-server

../../target/debug/echo-server:
	cargo build

install: ../../target/debug/echo-server
	# GitHub's runners don't have this directory by default.
	sudo mkdir -p /Library/PrivilegedHelperTools/
	sudo cp ../../target/debug/echo-server /Library/PrivilegedHelperTools/com.example.echo
	sudo cp com.example.echo.plist /Library/LaunchDaemons/
	sudo launchctl load /Library/LaunchDaemons/com.example.echo.plist

uninstall:
	sudo launchctl unload /Library/LaunchDaemons/com.example.echo.plist
	sudo rm /Library/PrivilegedHelperTools/com.example.echo
	sudo rm /Library/LaunchDaemons/com.example.echo.plist
