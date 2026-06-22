# sdd-spec-check

Verify that SDD specs and tests agree on rule IDs and text.

## Install

```sh
# npm (any Node project)
pnpm add -D sdd-spec-check
# or
npm i -D sdd-spec-check

# from source (Rust)
cargo install sdd-spec-check
```

The npm package downloads a prebuilt binary on install for Linux x64,
macOS x64, macOS arm64, and Windows x64.

## How it works

Specs are Markdown. Each rule has a stable ID.

```md
- [RULE-LOG-001] writes JSON to stdout
```

Tests reference the rule by ID and exact text.

```ts
it("RULE-LOG-001: writes JSON to stdout", () => { ... })
```

`sdd-spec-check` fails when:

1. A test uses an ID that no current spec defines
2. A current spec rule has no test
3. The test text doesn't match the spec

## Configure

Put `sdd-spec-check.config.toml` at the repo root:

```toml
specs = ["spec/**/*.md"]
tests = ["test/**/*.test.ts"]
```

It searches up from the current directory. Or pass `--config <path>`.

## Multi-language

List more globs. Add patterns if your test refs don't use plain quotes.
Each pattern needs named groups `id` and `text`.

```toml
tests = ["**/*.test.ts", "tests/**/*_test.py", "src/**/*_test.go"]

test_patterns = [
  '"(?<id>RULE-[A-Z0-9-]+):\s+(?<text>[^"]*?)"',
  "'(?<id>RULE-[A-Z0-9-]+):\s+(?<text>[^']*?)'",
  '`(?<id>RULE-[A-Z0-9-]+):\s+(?<text>[^`]*?)`',
]
```

Default patterns cover double, single, and backtick quotes.

## Strictness

```toml
on_empty_glob    = "warn"   # warn | error | silent
on_unknown_id    = "error"
on_missing_test  = "error"
on_text_mismatch = "error"
```

## Spec format

The default spec rule pattern is `- [RULE-X] text` in a Markdown bullet.
Change it with `spec_patterns` (same shape as `test_patterns`):

```toml
spec_patterns = ['^-\s+\[(?<id>RULE-[A-Z0-9-]+)\]\s+(?<text>.+)$']
```

## Text matching

A trailing period is stripped from both sides before comparing. This lets a spec bullet ending in `.` match a test string without one.

```md
- [RULE-LOG-001] writes JSON to stdout.
```
```ts
it("RULE-LOG-001: writes JSON to stdout", …)
```

Both reduce to `writes JSON to stdout` and the check passes.

## Exit codes

- `0` checks passed
- `1` checks failed
- `2` could not run (bad config, missing file)

## License

MIT.
