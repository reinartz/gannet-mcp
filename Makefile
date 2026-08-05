.PHONY: all build test clean rpm srpm

all: build

build:
	cargo build --release

test:
	cargo test
	cargo clippy -- -D warnings
	cargo fmt -- --check

clean:
	cargo clean
	rm -rf rpmbuild/

# RPM build targets (Fedora)
rpm:
	@if ! command -v rpmbuild &>/dev/null; then \
		echo "Error: rpmbuild not found. Install it with:"; \
		echo "  sudo dnf install rpm-build"; \
		exit 1; \
	fi
	rm -rf rpmbuild/
	mkdir -p rpmbuild/SOURCES rpmbuild/SPECS rpmbuild/RPMS rpmbuild/SRPMS rpmbuild/BUILD rpmbuild/BUILDROOT
	@set -e; \
	VERSION=$$(cargo metadata --no-deps --format-version 1 2>/dev/null | python3 -c "import sys,json; d=json.load(sys.stdin); print(d['packages'][0]['version'])"); \
	SOURCES="$(CURDIR)/rpmbuild/SOURCES"; \
	rm -rf "$$SOURCES"; \
	mkdir -p "$$SOURCES" rpmbuild/SPECS rpmbuild/RPMS rpmbuild/SRPMS rpmbuild/BUILD rpmbuild/BUILDROOT; \
	git archive --prefix="web-search-mcp-server-$$VERSION/" -o "$$SOURCES/web-search-mcp-server-$$VERSION.tar.gz" HEAD 2>/dev/null; \
	if [ "$$(tar tzf "$$SOURCES/web-search-mcp-server-$$VERSION.tar.gz" 2>/dev/null | wc -l)" -le 1 ]; then \
		tar czf "$$SOURCES/web-search-mcp-server-$$VERSION.tar.gz" \
			--exclude=target --exclude=rpmbuild --exclude=.git \
			--transform "s%^./%web-search-mcp-server-$$VERSION/%" \
			-C "$(CURDIR)" .; \
	fi; \
	rpmbuild -D "_topdir $(CURDIR)/rpmbuild" -ba web-search-mcp-server.spec

srpm:
	@if ! command -v rpmbuild &>/dev/null; then \
		echo "Error: rpmbuild not found. Install it with:"; \
		echo "  sudo dnf install rpm-build"; \
		exit 1; \
	fi
	@set -e; \
	VERSION=$$(cargo metadata --no-deps --format-version 1 2>/dev/null | python3 -c "import sys,json; d=json.load(sys.stdin); print(d['packages'][0]['version'])"); \
	SOURCES="$(CURDIR)/rpmbuild/SOURCES"; \
	rm -rf "$$SOURCES" rpmbuild/SPECS rpmbuild/SRPMS; \
	mkdir -p "$$SOURCES" rpmbuild/SPECS rpmbuild/SRPMS; \
	cp web-search-mcp-server.spec rpmbuild/SPECS/; \
	git archive --prefix="web-search-mcp-server-$$VERSION/" -o "$$SOURCES/web-search-mcp-server-$$VERSION.tar.gz" HEAD 2>/dev/null; \
	if [ "$$(tar tzf "$$SOURCES/web-search-mcp-server-$$VERSION.tar.gz" 2>/dev/null | wc -l)" -le 1 ]; then \
		tar czf "$$SOURCES/web-search-mcp-server-$$VERSION.tar.gz" \
			--exclude=target --exclude=rpmbuild --exclude=.git \
			--transform "s%^./%web-search-mcp-server-$$VERSION/%" \
			-C "$(CURDIR)" .; \
	fi; \
	rpmbuild -D "_topdir $(CURDIR)/rpmbuild" -bs web-search-mcp-server.spec

.PHONY: install-rpm-deps
install-rpm-deps:
	sudo dnf install -y rpm-build rust cargo rust-srpm-macros
