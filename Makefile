.PHONY: build release publish clean run test

build:
	cargo build

release:
	cargo build --release

publish: release
	@mkdir -p ~/bin
	cp target/release/sqlitex ~/bin/
	@echo "Installed sqlitex to ~/bin/sqlitex"

clean:
	cargo clean

run:
	cargo run -- $(ARGS)

test:
	cargo test
