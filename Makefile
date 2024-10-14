credits.html:
	cargo about generate about.hbs > credits.html

crossbuild:
	docker run --rm -v "$(PWD)":/home/rust/src messense/rust-musl-cross:aarch64-musl \
		sh -c "\
		  sudo apt-get update && sudo apt-get install -y libssl-dev pkg-config && \
		  cargo build --release --target aarch64-unknown-linux-musl" && \
		cp target/aarch64-unknown-linux-musl/release/r2sync ./r2sync
