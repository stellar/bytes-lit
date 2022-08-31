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
	cargo workspaces version --all --force '*' --allow-branch '*' --no-git-tag --no-git-push --yes custom 1.0.0

publish-verify:
	cargo +stable package
	pushd target/package/bytes-lit-* && \
		cargo +stable hack --feature-powerset build --locked && \
		cargo +stable hack --feature-powerset test --locked && \
		popd
	cargo +stable publish --locked --dry-run

publish: publish-verify
	cargo +stable publish --locked
	while ! cargo add --dry-run bytes-lit@$(cargo metadata --format-version 1 | jq -r '.packages[0].version') ; do echo waiting; sleep 10; done
