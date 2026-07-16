#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
grep -q 'struct Keyring' crates/aira-object/src/crypto.rs
grep -q 'identity sign' crates/aira-cli/src/main.rs || grep -q 'IdentityCommands::Sign' crates/aira-cli/src/main.rs
echo "OK: keyring + CLI sign present"
exit 0
