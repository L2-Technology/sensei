.PHONY: install-cli build-cli

install-cli:
	cargo build --bin senseicli
	cp target/debug/senseicli ~/bin/senseicli


