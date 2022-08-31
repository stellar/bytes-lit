all: check test

export RUSTFLAGS=-Dwarnings

doc: fmt
	cargo test --doc
	cargo doc --open

test: fmt
	cargo test

check: fmt
	cargo check

watch:
	cargo watch --clear --watch-when-idle --shell '$(MAKE)'

watch-doc:
	cargo +nightly watch --clear --watch-when-idle --shell '$(MAKE) doc CARGO_DOC_ARGS='

fmt:
	cargo fmt --all

clean:
	cargo clean

bump-version:
	cargo workspaces version --all --force '*' --allow-branch '*' --no-git-tag --no-git-push --yes custom $(VERSION)

publish-verify:
	perl -i -pe 's/.*git *=.*//go' Cargo.toml
	cargo publish --locked --dry-run
	$(MAKE) all

publish: publish-verify
	cargo publish --locked
	while ! cargo add --dry-run bytes-lit@$(cargo metadata --format-version 1 | jq -r '.packages[0].version') ; do echo waiting; sleep 10; done
