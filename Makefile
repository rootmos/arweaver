CARGO ?= cargo
DOCKER ?= docker
LOOM_PORT ?= 8000

.PHONY: test
test: integration-tests mutability-tests

.PHONY: integration-tests
integration-tests:
	$(CARGO) test --test integration_tests

.PHONY: mutability-tests
mutability-tests:
	env ARWEAVE_TARGET=http://localhost:$(LOOM_PORT)/arweave/ \
		LOOM_TARGET=http://localhost:$(LOOM_PORT)/loom/ \
		$(CARGO) test --test mutability_tests

.PHONY: start-loom
start-loom:
	$(DOCKER) run --detach --rm --publish=$(LOOM_PORT):8000 --name=loom \
		rootmos/loom@sha256:ed539ea6fbe23533bffd00166ebed8385fc400311ee339e5a9ac0d94247cc708

.PHONY: wait
wait:
	while ! curl --silent --fail http://localhost:$(LOOM_PORT)/arweave/info > /dev/null; do sleep 1; done

.PHONY: stop-loom
stop-loom:
	$(DOCKER) stop loom
