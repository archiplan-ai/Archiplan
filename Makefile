# Release tarballs (see RELEASE.md).
# Requires: cargo-zigbuild + zig for the linux/windows targets, rust targets
# x86_64-unknown-linux-gnu + aarch64-unknown-linux-gnu + x86_64-pc-windows-gnu,
# and an authenticated `gh` for publish. macOS arm64 builds natively.
#
# VERSION defaults to the version in crates/archi/Cargo.toml so the tarball
# name always matches what `archi --version` prints. Override only for
# throwaway test builds, e.g. make release VERSION=0.2.0-test1

VERSION ?= $(shell sed -n 's/^version = "\([^"]*\)".*/\1/p' crates/archi/Cargo.toml | head -1)

.PHONY: release release-macos-arm64 release-linux-x64 release-linux-arm64 release-windows-x64 pack publish

release: release-macos-arm64 release-linux-x64 release-linux-arm64 release-windows-x64
	@echo "built $(VERSION); now: make publish"

release-macos-arm64:
	cargo build -p archi --release --target aarch64-apple-darwin
	$(MAKE) pack PLATFORM=macos-arm64 \
	  BIN=target/aarch64-apple-darwin/release/archi

release-linux-x64:
	cargo zigbuild -p archi --release --target x86_64-unknown-linux-gnu
	$(MAKE) pack PLATFORM=linux-x64 \
	  BIN=target/x86_64-unknown-linux-gnu/release/archi

release-linux-arm64:
	cargo zigbuild -p archi --release --target aarch64-unknown-linux-gnu
	$(MAKE) pack PLATFORM=linux-arm64 \
	  BIN=target/aarch64-unknown-linux-gnu/release/archi

release-windows-x64:
	cargo zigbuild -p archi --release --target x86_64-pc-windows-gnu
	$(MAKE) pack PLATFORM=windows-x64 \
	  BIN=target/x86_64-pc-windows-gnu/release/archi.exe BIN_NAME=archi.exe \
	  INSTALL_SCRIPT=release/install.ps1

BIN_NAME ?= archi
INSTALL_SCRIPT ?= release/install.sh
pack:
	rm -rf dist/archi-$(VERSION)-$(PLATFORM)
	mkdir -p dist/archi-$(VERSION)-$(PLATFORM)
	install -m 755 $(BIN) dist/archi-$(VERSION)-$(PLATFORM)/$(BIN_NAME)
	cp $(INSTALL_SCRIPT) release/README.txt \
	  dist/archi-$(VERSION)-$(PLATFORM)/
	tar -C dist -czf dist/archi-$(VERSION)-$(PLATFORM).tar.gz \
	  archi-$(VERSION)-$(PLATFORM)
	cd dist && shasum -a 256 archi-$(VERSION)-$(PLATFORM).tar.gz \
	  > archi-$(VERSION)-$(PLATFORM).tar.gz.sha256

publish:
	gh release create v$(VERSION) --title "archi $(VERSION)" --generate-notes \
	  dist/archi-$(VERSION)-*.tar.gz dist/archi-$(VERSION)-*.tar.gz.sha256
