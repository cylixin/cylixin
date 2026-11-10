#!/usr/bin/env bash
# Runs one CI check on the compiler crate and turns its failures into GitHub
# annotations, so problems show up inline on the pull request's "Files changed" tab.
#
#   .github/scripts/rust-check.sh fmt|clippy|test|examples
#
# Works locally too: annotations are printed as plain "::error file=..." lines.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
CRATE_DIR="cylixin_compiler"           # crate path relative to the repo root
cd "$ROOT/$CRATE_DIR" || exit 1

# Escape text for a workflow command (GitHub requires %, CR and LF to be encoded).
esc() { local s="$1"; s="${s//'%'/'%25'}"; s="${s//$'\r'/'%0D'}"; s="${s//$'\n'/'%0A'}"; printf '%s' "$s"; }
annotate() { # level file line col title message
  printf '::%s file=%s,line=%s,col=%s,title=%s::%s\n' "$1" "$2" "$3" "$4" "$(esc "$5")" "$(esc "$6")"
}

check_fmt() {
  local out status
  out="$(cargo fmt --check 2>&1)"; status=$?
  printf '%s\n' "$out"
  # Lines look like: "Diff in /abs/path/cylixin_compiler/src/main.rs:12:"
  printf '%s\n' "$out" | sed -n 's#^Diff in \(.*\):\([0-9][0-9]*\):$#\1 \2#p' |
  while read -r file line; do
    annotate error "${file#"$ROOT/"}" "$line" 1 "rustfmt" \
      "This code is not formatted. Run \`cargo fmt\` in $CRATE_DIR and commit the result."
  done
  return $status
}

check_clippy() {
  local json status
  json="$(cargo clippy --all-targets --message-format=json -- -D warnings 2>/dev/null)"; status=$?
  # Human-readable output for the log.
  printf '%s\n' "$json" | jq -r 'select(.reason == "compiler-message") | .message.rendered // empty'
  # One annotation per primary span. Paths from cargo are relative to the crate.
  printf '%s\n' "$json" | jq -r --arg dir "$CRATE_DIR" '
    select(.reason == "compiler-message") | .message
    | select(.level == "error" or .level == "warning") as $m
    | ($m.spans[] | select(.is_primary)) as $s
    | [ (if $m.level == "error" then "error" else "warning" end),
        "\($dir)/\($s.file_name)", ($s.line_start|tostring), ($s.column_start|tostring),
        ($m.code.code // "compiler"), $m.message ] | @tsv' |
  sort -u |  # the bin and test targets report the same diagnostic twice
  while IFS=$'\t' read -r level file line col code msg; do
    annotate "$level" "$file" "$line" "$col" "$code" "$msg"
  done
  if [ $status -ne 0 ]; then
    echo "Clippy failed. Reproduce locally with: cargo clippy --all-targets -- -D warnings"
  fi
  return $status
}

check_test() {
  local log status
  log="$(mktemp)"
  cargo test --color=never 2>&1 | tee "$log"; status=${PIPESTATUS[0]}
  # Panics look like: "thread 'lexer::tests::foo' panicked at src/lexer/lexer.rs:300:9:"
  # Collect each failing test's location, then its first message line, as one annotation.
  sed -n "s#^thread '\([^']*\)' \(([0-9]*) \)\{0,1\}panicked at \([^:]*\):\([0-9]*\):\([0-9]*\):\{0,1\}\$#\1\t\3\t\4\t\5#p" "$log" |
  while IFS=$'\t' read -r test file line col; do
    msg="$(grep -A1 -F "thread '$test'" "$log" | sed -n 2p)"
    annotate error "$CRATE_DIR/$file" "$line" "$col" "test failed: $test" "${msg:-see the test log}"
  done
  rm -f "$log"
  return $status
}

check_examples() {
  # Build every example end to end (lex, parse, check, IR, clang link), then run the
  # ones that don't wait for keyboard input.
  local bin="$ROOT/$CRATE_DIR/target/debug/cylixin" out dir status=0
  cargo build --quiet || return 1
  out="$(mktemp -d)"
  for src in code_examples/*.cyx; do
    name="$(basename "$src" .cyx)"
    echo "::group::build $src"
    if ! (cd "$out" && "$bin" build "$ROOT/$CRATE_DIR/$src" -o "$name"); then
      annotate error "$CRATE_DIR/$src" 1 1 "example failed to build" \
        "\`cylixin build $src\` failed. See the 'Build and run examples' step log."
      status=1
    fi
    echo "::endgroup::"
  done
  if [ -x "$out/demo" ]; then
    echo "::group::run demo"
    if ! timeout 20 "$out/demo" </dev/null; then
      annotate error "$CRATE_DIR/code_examples/demo.cyx" 1 1 "demo failed at runtime" \
        "demo.cyx compiled but exited with an error. See the 'Build and run examples' step log."
      status=1
    fi
    echo "::endgroup::"
  fi
  rm -rf "$out"
  return $status
}

case "${1:-}" in
  fmt) check_fmt ;;
  clippy) check_clippy ;;
  test) check_test ;;
  examples) check_examples ;;
  *) echo "usage: $0 fmt|clippy|test|examples" >&2; exit 2 ;;
esac
