MODULES := $(wildcard modules/*)

help: ## Display this help screen
	@grep -h -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}'

$(MODULES):
	@cargo build \
          --manifest-path=./$@/Cargo.toml \
	  --color=always \
	  --release \
	  --target wasm32-unknown-unknown \
	  2>&1

# /*	  -Z build-std=core,alloc,panic_abort \
# 	  -Z build-std-features=panic_immediate_abort \ */

modules: $(MODULES)

test: modules ## Run the module tests
	@cargo test \
	  --manifest-path=./hatchery/Cargo.toml \
	  --color=always\
	  2>&1

.PHONY: all $(MODULES)
