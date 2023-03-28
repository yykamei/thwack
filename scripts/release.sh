#!/bin/bash

set -xeuo pipefail

# Check whether the version is described in the source code.
grep "$VERSION" Cargo.toml

cargo publish --dry-run --allow-dirty
cargo package --list --allow-dirty
gh release list
cat tmp/notes.txt

if [[ "$APPLY" == "true" ]]; then
  gh release create "v$VERSION" --generate-notes
  cargo publish
fi
