run:
	cargo run --

test:
	cargo test

build:
	cargo build

install:
	cargo install --path .
	confet &

watch:
	cargo watch -x 'run --'

# Bump patch version, commit, tag, push, and publish
release:
	@set -e; \
	VERSION=$$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/'); \
	MAJOR=$$(echo $$VERSION | cut -d. -f1); \
	MINOR=$$(echo $$VERSION | cut -d. -f2); \
	PATCH=$$(echo $$VERSION | cut -d. -f3); \
	NEW_VERSION="$$MAJOR.$$MINOR.$$((PATCH + 1))"; \
	sed -i.bak "s/^version = \"$$VERSION\"/version = \"$$NEW_VERSION\"/" Cargo.toml; \
	rm -f Cargo.toml.bak; \
	grep -q "^version = \"$$NEW_VERSION\"" Cargo.toml; \
	cargo check --quiet; \
	git add Cargo.toml Cargo.lock; \
	git commit -m "v$$NEW_VERSION"; \
	git tag "v$$NEW_VERSION"; \
	git push && git push --tags; \
	cargo publish; \
	echo "Released v$$NEW_VERSION"
	confet
