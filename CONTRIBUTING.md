# Contributing to `meta-magic_mount-rs`

Thank you for contributing!

This document explains how to prepare your environment, make changes, and submit pull requests that are easy to review.

---

## Table of contents

- [Code of conduct](#code-of-conduct)
- [Ways to contribute](#ways-to-contribute)
- [Development setup](#development-setup)
- [Build and run](#build-and-run)
- [Project layout](#project-layout)
- [Coding guidelines](#coding-guidelines)
- [Testing and validation](#testing-and-validation)
- [Commit and PR guidelines](#commit-and-pr-guidelines)
- [Review expectations](#review-expectations)

---

## Code of conduct

- Be respectful and constructive in issues and pull requests.
- Assume good intent and focus on technical outcomes.

---

## Ways to contribute

You can help by:

- Reporting bugs (with reproduction steps and logs).
- Proposing or implementing features.
- Improving docs, examples, and developer tooling.
- Reviewing open pull requests.

For larger changes (new behavior, refactors, architecture-impacting work), open an issue first so we can align on scope.

---

## Development setup

Based on `docs/README_en.md`, the typical toolchain is:

- Rust nightly toolchain
- Android NDK
- `cargo-ndk`
- Node.js / npm
- `pnpm` (for `webui`)

Recommended environment variables:

```bash
export ANDROID_NDK_HOME=<path/to/ndk>
export ANDROID_NDK_ROOT=$ANDROID_NDK_HOME
```

---

## Build and run

From the repository root:

```bash
cargo xtask b
```

Expected artifact:

- `output/magic_mount_rs.zip`

---

## Project layout

- `src/`: core Rust implementation.
- `xtask/`: build orchestration and helper tasks.
- `module/`: packaging scripts and module assets.
- `webui/`: frontend UI for configuration.
- `docs/`: user-facing documentation.

Keep changes scoped to the minimum set of files required for the problem.

---

## Coding guidelines

### Rust

Before submitting, run:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
```

Guidelines:

- Prefer small, focused functions and clear naming.
- Avoid unrelated refactors in feature/fix PRs.
- Do not add new dependencies unless necessary.
- Preserve backward compatibility unless the PR explicitly documents a breaking change.

### WebUI

In `webui/`:

```bash
pnpm install
pnpm lint
pnpm build
```

Follow the existing ESLint/Prettier conventions.

---

## Testing and validation

At minimum, validate the paths touched by your change.

Suggested checks:

```bash
cargo check
```

When applicable, include:

- Manual verification steps.
- Environment/device details.
- Before/after behavior summary.

If fixing a bug, add or document a reproducible regression check whenever possible.

---

## Commit and PR guidelines

### Commits

- Use clear commit messages that explain intent.
- Keep commits atomic and logically grouped.

### Pull requests

Please include:

1. **What changed** (summary)
2. **Why** (problem/context)
3. **How it was tested** (commands + results)
4. **Compatibility impact** (if any)
5. **Screenshots** (for visible WebUI changes)

Also:

- Link related issues (for example, `Fixes #123`).
- Keep PRs focused: one concern per PR when possible.

---

## Review expectations

- Maintainers may ask for additional tests, simplification, or scope reduction.
- Prefer follow-up commits over force-push during active review (unless asked otherwise).
- If feedback is unclear, ask for clarification directly in the PR.

Thanks again for helping improve `meta-magic_mount-rs`.

