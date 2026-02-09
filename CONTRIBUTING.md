# Contributing

Thanks for considering contributing.

## Quick guidelines
- Keep changes focused (one issue per PR).
- Prefer small PRs that are easy to review.
- Match existing code style.
- Avoid introducing new UX/features unless discussed first.

## Development setup
```powershell
cargo build
cargo run
```

## Before opening a PR
- Run `cargo fmt` (if you use rustfmt).
- Run `cargo check`.
- If you changed installer files, verify the Inno script compiles.

## What to include in the PR
- A short description of the change and why.
- Screenshots/GIFs for UI changes.
- Notes about any breaking changes.

## Reporting bugs
Please use the Bug Report issue template and include:
- Windows version
- Steps to reproduce
- Expected vs actual behavior
- Any logs/messages (avoid sensitive data)
