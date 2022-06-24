all:
	@cargo b --release \
	         --target wasm32-unknown-unknown \
	         -Z build-std=std,panic_abort \
	         -Z build-std-features=panic_immediate_abort
