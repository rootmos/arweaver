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

.PHONY: build
build:
	$(CARGO) build

.PHONY: start-loom
start-loom:
	$(DOCKER) run --detach --rm --publish=$(LOOM_PORT):8000 --name=loom \
		rootmos/loom@sha256:29c71d1aad4a116b1b3af656feaa647b2524f315192dd02dee6eeeb84dc77ca6

.PHONY: wait
wait:
	while ! curl --silent --fail http://localhost:$(LOOM_PORT)/arweave/info > /dev/null; do sleep 1; done

.PHONY: stop-loom
stop-loom:
	$(DOCKER) stop loom
