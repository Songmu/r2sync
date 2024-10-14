ver = v0.0.2

credits.html:
	cargo about generate about.hbs > credits.html

crossbuild:
	rm -rf dist
	mkdir dist
	docker run --rm -v "$(PWD)":/home/rust/src -w /home/rust/src rust:latest \
		sh -c "\
		  rustup target add aarch64-unknown-linux-gnu && \
		  apt-get update && apt-get install -y gcc-aarch64-linux-gnu && \
		  export RUSTFLAGS='-C strip=symbols' && \
		  cargo build --release --target aarch64-unknown-linux-gnu && \
		  mkdir dist/r2sync-${ver}-linux-arm64 && \
		  mv target/aarch64-unknown-linux-gnu/release/r2sync ./dist/r2sync-${ver}-linux-arm64/r2sync"
