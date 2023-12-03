# some build commands for local development

.PHONY: wasm

build:
	cargo build --release --no-default-features

wasm:
	cargo build --release --no-default-features --target wasm32-unknown-unknown
	wasm-bindgen --no-typescript --out-name bevy_game --out-dir wasm --target web target/wasm32-unknown-unknown/release/bevy_officespace.wasm
	mkdir -p wasm/assets || true
	cp assets/* wasm/assets || true
	cd wasm/ && python3 -m http.server
