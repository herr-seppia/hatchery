MODULES := $(wildcard modules/*)

help: ## Display this help screen
	@grep -h -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}'

$(MODULES):
	cargo build \
          --manifest-path=./$@/Cargo.toml \
	  --color=always \
	  --target wasm32-unknown-unknown

modules: $(MODULES)

test: modules ## Run the module tests
	cargo test \
	  --manifest-path=./hatchery/Cargo.toml \
	  --color=always

.PHONY: all $(MODULES)
