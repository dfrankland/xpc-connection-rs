all: build

.PHONY: build
build:
	cargo build

.PHONY: test
test: install
	cd client && cargo run

.PHONY: install
install: build
	sudo mkdir -p /Library/PrivilegedHelperTools/
	sudo cp ../target/debug/echo-server /Library/PrivilegedHelperTools/echo-daemon
	sudo cp echo-server/echo-daemon.plist /Library/LaunchDaemons/
	sudo launchctl load /Library/LaunchDaemons/echo-daemon.plist

	sudo mkdir -p /usr/local/bin
	sudo cp echo-server/echo-agent.plist /Library/LaunchAgents/
	sudo cp ../target/debug/echo-server /usr/local/bin/echo-agent
	launchctl load /Library/LaunchAgents/echo-agent.plist

.PHONY: uninstall
uninstall:
	sudo launchctl unload /Library/LaunchDaemons/echo-daemon.plist
	sudo rm -f /Library/LaunchDaemons/echo-daemon.plist
	sudo rm -f /Library/PrivilegedHelperTools/echo-daemon

	launchctl unload /Library/LaunchAgents/echo-agent.plist
	sudo rm -f /Library/LaunchAgents/echo-agent.plist
	sudo rm -f /usr/local/bin/echo-agent
