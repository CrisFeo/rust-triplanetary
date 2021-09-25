.PHONY: build
build:
	./scripts/build

.PHONY: run
run: build
	./scripts/run

.PHONY: watch
watch:
	./scripts/watch

.PHONY: doc
doc:
	./scripts/doc
