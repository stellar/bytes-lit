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
	cargo release --workspace --no-push --no-tag --execute --no-confirm
