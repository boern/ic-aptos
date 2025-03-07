#!/usr/bin/make

.DEFAULT_GOAL: help

help: ## Show this help
	@printf "\033[33m%s:\033[0m\n" 'Available commands'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  \033[32m%-18s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# ----------------------------------------------------------------------------------------------------------------------

.PHONY: build
build: ## Build for wasm target
	cargo build --release --target wasm32-unknown-unknown

.PHONY: check
check: build ## Check the incompatible deps
	@if cargo tree 2>/dev/null | grep -q wasm-bindgen; then \
		cargo tree | grep wasm-bindgen; \
	else \
		echo "no found wasm-bindgen"; \
	fi

	@if cargo tree 2>/dev/null | grep -q tokio; then \
		cargo tree | grep tokio; \
	else \
		echo "no found tokio"; \
	fi

.PHONY: clean
clean: ## Cleanup
	cargo clean

%::
	@true
