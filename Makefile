.PHONY: build
build:
	./scripts/build

.PHONY: run
run: build
	./scripts/run

.PHONY: check
check:
	./scripts/check

.PHONY: watch
watch:
	./scripts/watch
