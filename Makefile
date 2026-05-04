ROOT := $(CURDIR)

.PHONY: build-expect test-upstream build-upstream build-upstream-report build-wasix prepare-webc-assets package-build test-cli deploy

build-expect:
	RUSTFLAGS="-C linker=rust-lld" cargo build -p pjdfstest --target aarch64-unknown-linux-musl

test-upstream: build-expect
	@set -e; \
	find "$(ROOT)/test_scenarious" -mindepth 2 -maxdepth 2 -name '*.t' | sort | while read -r test_file; do \
		rel="$${test_file#$(ROOT)/test_scenarious/}"; \
		group="$${rel%%/*}"; \
		echo "==> $${rel}"; \
		docker run --rm \
			--volume "$(ROOT)/target/aarch64-unknown-linux-musl/debug/pjdfstest:/app/pjdfstest:ro" \
			--volume "$(ROOT)/test_scenarious/misc.sh:/app/test/misc.sh:ro" \
			--volume "$(ROOT)/test_scenarious/conf:/app/test/conf:ro" \
			--volume "$(ROOT)/test_scenarious/$${group}:/app/test/$${group}:ro" \
			--workdir /work \
			ubuntu:24.04 \
			bash -c 'bash "/app/test/$$1"' _ "$${rel}"; \
	done

build-upstream:
	$(MAKE) test-upstream > report.txt

build-upstream-report: build-upstream
	cargo run -p reportter -- report.txt
	open index.html

build-wasix:
	cargo wasix build --release -p client-http
	cargo wasix build --release -p client-cli
	cargo wasix build --release -p pjdfstest
	cargo wasix build --release -p uname

prepare-webc-assets: build-wasix
	rm -rf package-assets/app
	rm -rf package-assets/wasix
	mkdir -p package-assets/app/tests
	mkdir -p package-assets/wasix
	cp -R test_scenarious/. package-assets/app/tests/
	cp test_scenarious/misc.sh package-assets/app/misc.sh
	cp test_scenarious/conf package-assets/app/conf
	cp target/wasm32-wasmer-wasi/release/pjdfstest.rustc.wasm package-assets/app/pjdfstest
	cp target/wasm32-wasmer-wasi/release/uname.rustc.wasm package-assets/wasix/uname
	chmod +x package-assets/app/pjdfstest
	chmod +x package-assets/wasix/uname

package-build: prepare-webc-assets
	rm -f /tmp/pjdfstest.webc
	wasmer package build -o /tmp/pjdfstest.webc

test-cli: prepare-webc-assets
	wasmer run --entrypoint client-cli .

deploy: prepare-webc-assets
	wasmer deploy --bump
