CARGO = cargo

UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
	CARGO += --config 'build.rustdocflags = ["-C", "link-args=-framework CoreFoundation -framework Security"]'
endif

help: ## Display this help screen
	@grep -h \
		-E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

clippy: ## Run clippy checks over all workspace members and formatted correctly
	@cargo check
	@cargo fmt --all -- --check
	@cargo clippy --all-targets -- -D warnings

fix: ## Automatically apply lint suggestions. This flag implies `--no-deps` and `--all-targets`
	@cargo clippy --fix

test: ## Run tests for all the workspace members
	@cargo test --release --all

.PHONY: clippy fmt test