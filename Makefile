ROOT := $(CURDIR)

.PHONY: build-expect test-upstream

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
