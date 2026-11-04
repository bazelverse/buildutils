# MIT License
#
# Make will use bash instead of sh
SHELL := /usr/bin/env bash

.PHONY: help
help:
	@echo ' help:'
	@echo '    make build   	Builds the code base incrementally (fast) for dev.'
	@echo '    make current   	Builds the current target incrementally (fast) defined in current.txt.'
	@echo '    make doc   		Builds documentation for the project.'
	@echo '    make format   	Formats call code according to cargo fmt style.'
	@echo '    make lint   	Lints and formats the code of the project.'
	@echo '    make fix   		Fixes linting issues as reported by clippy.'
	@echo '    make test   	Tests across all crates with Bazel.'
	@echo '    make test_cargo	Tests across all crates with Cargo, staging binaries first.'


.PHONY: build
build:
	@source scripts/build.sh

.PHONY: current
current:
	@source scripts/current.sh

.PHONY: doc
doc:
	@source scripts/doc.sh

.PHONY: format
format:
	@source scripts/format.sh

.PHONY: lint
lint:
	@source scripts/lint.sh

.PHONY: fix
fix:
	@source scripts/fix.sh

.PHONY: test
test:
	@source scripts/test.sh

.PHONY: test_cargo
test_cargo:
	@source scripts/test_cargo.sh

