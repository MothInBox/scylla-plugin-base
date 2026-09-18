# The path where Cargo builds the file
BUILD_PATH := target/wasm32-unknown-unknown/release/plugin_royalroad.wasm
# The path where your application looks for plugins
DEST_DIR := $(HOME)/.config/scylla-reader/plugins
# The exact name your registry code expects
FINAL_NAME := plugin-royalroad.wasm

build:
	cargo build --target wasm32-unknown-unknown --release
	@mkdir -p $(DEST_DIR)
	mv $(BUILD_PATH) $(DEST_DIR)/$(FINAL_NAME)
	@echo "Plugin built and moved to $(DEST_DIR)/$(FINAL_NAME)"
