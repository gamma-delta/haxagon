main :
	@echo "use a more specific command"

# You'll probably have to configure this for your own use-case;
# I happen to be using cygwin on Windows.
android : 
	docker run --rm \
		-v $(shell pwd | sed 's!cygdrive/!!'):/root/src \
		-v $(USERPROFILE)/.cargo/registry:/tmp/registry \
		-w /root/src notfl3/cargo-apk \
		cargo quad-apk build --release
