#
# SPDX-License-Identifier: MIT
# Copyright (c) "2025" . The buildutils Authors and Contributors. All Rights Reserved.
#

# bin/sh
set -o errexit
set -o nounset
set -o pipefail

echo "=============="
echo "Lint targets "
echo "=============="
# https://users.rust-lang.org/t/ive-ran-clippy-on-100k-crates/117127
# https://www.reddit.com/r/rust/comments/a4wblu/how_to_configure_clippy_to_be_as_annoying_as/
cargo clippy --workspace --all-targets -- -Wclippy::all

echo "=============="
echo "Format targets "
echo "=============="
# Bazel file formatting (Installed via homebrew)
# https://github.com/bazelbuild/buildtools
command buildifier -r MODULE.bazel BUILD.bazel
command buildifier -r build crates/*

# Rust code formatting
# https://github.com/rust-lang/rustfmt
command cargo fmt --all


# Check for uncommited changes before building and testing.
# It is possible that either image update or fie format changed some files,
# so it is important to check for uncommited changes before continuing.
if [[ $(git status --porcelain | wc -l) -gt 0 ]];
then
  #
  echo "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
  echo "Uncommited changes found; commit first, then run script again"
  echo "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
  # Full stop
  exit 1
fi
