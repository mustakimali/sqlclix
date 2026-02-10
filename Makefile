TARGET_DIR ?= $(or $(CARGO_TARGET_DIR),target)

.PHONY: build release publish clean run test

build:
	cargo build

release:
	cargo build --release

publish: release
	@mkdir -p ~/bin
	cp $(TARGET_DIR)/release/sqlclix ~/bin/
	@echo "Installed sqlclix to ~/bin/sqlclix"

clean:
	cargo clean

run:
	cargo run -- $(ARGS)

test:
	cargo test
