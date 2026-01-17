OUTPUT=langs-detector

build:
	clear
	cargo build
release:
	clear
	cargo build --release

dev:
	make build
	clear
	./target/debug/$(OUTPUT)
run:
	make release
	clear
	./target/release/$(OUTPUT)
