# Contributing

## Primary rule
- **No god files**: one responsibility per module. Split a file into a folder module as soon as it mixes concerns
  or grows past ~250 lines (pure data tables such as `i18n.rs` excepted).
- **Strict DRY**: never write the same logic twice; factor it into a helper and reuse existing ones first.

## Before a pull request
- `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test` must pass (CI enforces them).
- Business rules go in `src/domain.rs` with unit tests; Win32 code stays thin.
- New UI text: add a `T` variant and translate it in all 25 languages of `src/i18n.rs`.
