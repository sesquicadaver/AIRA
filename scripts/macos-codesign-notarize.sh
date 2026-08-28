#!/usr/bin/env bash
# QUEUE #148 / Analyze-178 — macOS codesign + notarization helper (production path).
#
# Default mode is --dry-run: print the exact commands that would run and exit 0
# without calling codesign/notarytool (works on Linux CI and macOS without Apple ID).
# Real signing requires Darwin + Developer ID credentials (see docs/desktop-packaging-macos.md).
#
# Out of scope: App Store / Mac App Store submission.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

DRY_RUN=1
APP_PATH=""
IDENTITY="${CODESIGN_IDENTITY:-Developer ID Application: EXAMPLE (TEAMID)}"
BUNDLE_ID="${NOTARY_BUNDLE_ID:-ai.aira.desktop}"
KEYCHAIN_PROFILE="${NOTARY_KEYCHAIN_PROFILE:-aira-notary}"
STAPLE=1

usage() {
  cat <<'EOF'
Usage: scripts/macos-codesign-notarize.sh [--dry-run|--execute] --app PATH [options]

Codesign AIRA Desktop.app, submit for notarization, and optionally staple the ticket.

  --dry-run     Print planned commands only (default; no Apple tools required)
  --execute     Run codesign / notarytool / stapler (requires Darwin + credentials)
  --app PATH    Path to AIRA Desktop.app (required)
  --identity S  codesign identity (default: $CODESIGN_IDENTITY or placeholder)
  --profile P   notarytool keychain profile (default: $NOTARY_KEYCHAIN_PROFILE)
  --no-staple   Skip xcrun stapler staple after notarization

Environment:
  CODESIGN_IDENTITY          Developer ID Application identity string
  NOTARY_KEYCHAIN_PROFILE    notarytool keychain-profile name
  NOTARY_BUNDLE_ID           Bundle id (default: ai.aira.desktop)

Examples:
  ./scripts/macos-codesign-notarize.sh --dry-run \
    --app release/desktop/aira-desktop-macos-0.1.0-arm64/AIRA\ Desktop.app

  CODESIGN_IDENTITY="Developer ID Application: …" \
  NOTARY_KEYCHAIN_PROFILE=aira-notary \
    ./scripts/macos-codesign-notarize.sh --execute --app /path/to/AIRA\ Desktop.app
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run) DRY_RUN=1; shift ;;
    --execute) DRY_RUN=0; shift ;;
    --app) APP_PATH="$2"; shift 2 ;;
    --identity) IDENTITY="$2"; shift 2 ;;
    --profile) KEYCHAIN_PROFILE="$2"; shift 2 ;;
    --no-staple) STAPLE=0; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown arg: $1" >&2; usage >&2; exit 2 ;;
  esac
done

if [[ -z "$APP_PATH" ]]; then
  echo "error: --app PATH is required" >&2
  usage >&2
  exit 2
fi

# Normalize to absolute path for display / execution.
if [[ "$APP_PATH" != /* ]]; then
  APP_PATH="$ROOT/$APP_PATH"
fi

ZIP_PATH="${APP_PATH%/}.notarize.zip"

run_or_echo() {
  if [[ "$DRY_RUN" -eq 1 ]]; then
    printf '+ %s\n' "$*"
  else
    "$@"
  fi
}

echo "== macOS codesign / notarize helper (QUEUE #148) =="
echo "app:      $APP_PATH"
echo "identity: $IDENTITY"
echo "profile:  $KEYCHAIN_PROFILE"
echo "bundle:   $BUNDLE_ID"
if [[ "$DRY_RUN" -eq 1 ]]; then
  echo "mode:     dry-run (no Apple tools invoked)"
else
  echo "mode:     execute"
  if [[ "$(uname -s)" != "Darwin" ]]; then
    echo "error: --execute requires Darwin (macOS)" >&2
    exit 1
  fi
  if [[ ! -d "$APP_PATH" ]]; then
    echo "error: app bundle not found: $APP_PATH" >&2
    exit 1
  fi
fi

# Deep sign nested Mach-O then the .app; hardened runtime for Gatekeeper.
run_or_echo codesign --force --deep --options runtime \
  --sign "$IDENTITY" \
  --timestamp \
  "$APP_PATH"

run_or_echo codesign --verify --deep --strict --verbose=2 "$APP_PATH"

# Zip for notarytool (ditto preserves resource forks / xattrs on macOS).
if [[ "$DRY_RUN" -eq 1 ]]; then
  printf '+ ditto -c -k --keepParent %q %q\n' "$APP_PATH" "$ZIP_PATH"
  printf '+ xcrun notarytool submit %q --keychain-profile %q --wait\n' \
    "$ZIP_PATH" "$KEYCHAIN_PROFILE"
else
  ditto -c -k --keepParent "$APP_PATH" "$ZIP_PATH"
  xcrun notarytool submit "$ZIP_PATH" --keychain-profile "$KEYCHAIN_PROFILE" --wait
fi

if [[ "$STAPLE" -eq 1 ]]; then
  run_or_echo xcrun stapler staple "$APP_PATH"
  run_or_echo xcrun stapler validate "$APP_PATH"
fi

if [[ "$DRY_RUN" -eq 1 ]]; then
  echo "dry-run complete — see docs/desktop-packaging-macos.md §Codesign / notarization"
  exit 0
fi

echo "notarization path complete for $APP_PATH"
