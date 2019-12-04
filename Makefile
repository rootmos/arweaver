test:
	cargo test --test integration_tests
	env ARWEAVE_TARGET=http://localhost:8000/arweave/ \
		LOOM_TARGET=http://localhost:8000/loom/ \
		cargo test --test mutability_tests

.PHONY: test
