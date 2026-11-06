name = r2sync
ver = v0.0.5

credits.html:
	cargo about generate about.hbs > credits.html

.PHONY: all
all: clean credits.html linux-arm64 linux-amd64 darwin-arm64 darwin-amd64
	./author/dist.sh

.PHONY: upload
upload: all
	env GITHUB_TOKEN=$$(gh auth token) ghr ${ver} dist/

.PHONY: release
release: upload
	git tag -fa v0 -m "Release v0"
	git push origin v0 --force

.PHONY: linux-arm64
linux-arm64:
	docker run --rm --platform linux/arm64 -v "$(PWD)":/home/rust/src -w /home/rust/src rust:latest \
		sh -c "\
		  rustup target add aarch64-unknown-linux-gnu && \
		  apt-get update && apt-get install -y gcc-aarch64-linux-gnu && \
		  export RUSTFLAGS='-C strip=symbols' && \
		  cargo build --release --target aarch64-unknown-linux-gnu && \
		  mkdir -p dist/${name}_${ver}_linux_arm64 && \
		  mv target/aarch64-unknown-linux-gnu/release/${name} ./dist/${name}_${ver}_linux_arm64/${name}"

.PHONY: linux-amd64
linux-amd64:
	docker run --rm --platform linux/amd64 -v "$(PWD)":/home/rust/src -w /home/rust/src rust:latest \
		sh -c "\
		  rustup target add x86_64-unknown-linux-gnu && \
		  apt-get update && apt-get install -y gcc && \
		  export RUSTFLAGS='-C strip=symbols' && \
		  cargo build --release --target x86_64-unknown-linux-gnu && \
		  mkdir -p dist/${name}_${ver}_linux_amd64 && \
		  mv target/x86_64-unknown-linux-gnu/release/${name} ./dist/${name}_${ver}_linux_amd64/${name}"

.PHONE: darwin-arm64
darwin-arm64:
	cargo build --release --target aarch64-apple-darwin
	mkdir -p dist/${name}_${ver}_darwin_arm64
	mv target/aarch64-apple-darwin/release/${name} ./dist/${name}_${ver}_darwin_arm64/${name}

.PHONE: darwin-amd64
darwin-amd64:
	cargo build --release --target x86_64-apple-darwin
	mkdir -p dist/${name}_${ver}_darwin_amd64
	mv target/x86_64-apple-darwin/release/${name} ./dist/${name}_${ver}_darwin_amd64/${name}

.PHONY: clean
clean:
	rm -rf dist
	rm -f credits.html
