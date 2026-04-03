APP_NAME := AgentView
BUNDLE_DIR := src-tauri/target/release/bundle/macos
APP_BUNDLE := $(BUNDLE_DIR)/$(APP_NAME).app
INSTALL_DIR := /Applications

.PHONY: build install clean dev

build:
	npm run tauri build
	@echo ""
	@echo "Built: $(APP_BUNDLE)"
	@du -sh "$(APP_BUNDLE)"

install: build
	@echo "Installing to $(INSTALL_DIR)/$(APP_NAME).app ..."
	rm -rf "$(INSTALL_DIR)/$(APP_NAME).app"
	cp -R "$(APP_BUNDLE)" "$(INSTALL_DIR)/"
	@echo "Done. Launch from Spotlight or: open -a $(APP_NAME)"

clean:
	rm -rf src-tauri/target/release dist
	@echo "Cleaned build artifacts"

dev:
	npm run tauri dev
