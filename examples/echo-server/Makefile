CFLAGS+=-O2

all: com.example.echo

com.example.echo: server.c
	clang -o com.example.echo $(CFLAGS) server.c

clean:
	rm com.example.echo

install:
	cp com.example.echo /Library/PrivilegedHelperTools/
	cp com.example.echo.plist /Library/LaunchDaemons/
	launchctl load /Library/LaunchDaemons/com.example.echo.plist

uninstall:
	launchctl unload /Library/LaunchDaemons/com.example.echo.plist
	rm /Library/PrivilegedHelperTools/com.example.echo