# Repository Guidelines

## Project Structure & Module Organization

This repository is a Cargo workspace. Backend Rust code belongs in `backend/src/`, organized by responsibility; the planned Qt frontend belongs in `frontend/qt/`. Keep automated checks under `tests/` or beside their implementation according to the chosen framework, and checked-in resources under `assets/`. Avoid adding generated output, dependency caches, or local secrets to version control.

## Build, Test, and Development Commands

Use `cargo build --release --locked`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo fmt --all -- --check` from the repository root. Document additional frontend and packaging commands in `docs/development.md`.

## Coding Style & Naming Conventions

Follow the formatter and linter configured for the chosen language. Until those tools are added, use consistent indentation within each file, descriptive names, and small modules with clear responsibilities. Use conventional lowercase names for directories; follow the language ecosystem’s standard naming for files and identifiers. Add formatting and linting configuration alongside the first implementation rather than relying on undocumented personal settings.

## Testing Guidelines

Rust unit tests live beside the implementation. Run the full suite with `cargo test --workspace`; document additional integration setup in `docs/development.md`.

## Commit & Pull Request Guidelines

There is no repository history from which to infer a commit convention. Use concise Conventional Commit subjects, for example `feat: add radial key layout` or `fix: handle empty input`. Keep changes focused. Pull requests should explain the purpose, summarize notable changes, list validation performed, link related issues when applicable, and include screenshots for user-visible interface changes.

## Security & Configuration

Keep credentials, tokens, and machine-specific configuration out of commits. Provide safe example configuration with placeholder values when needed, and document required environment variables without including real secrets.
