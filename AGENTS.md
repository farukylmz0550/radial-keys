# Repository Guidelines

## Project Structure & Module Organization

This repository is currently empty: no application source, tests, assets, or build configuration are present. Keep future code organized by purpose (for example, `src/` for implementation, `tests/` for automated checks, and `assets/` for checked-in resources). Update this guide when the project structure is established; avoid adding generated output, dependency caches, or local secrets to version control.

## Build, Test, and Development Commands

There are no project-specific build, test, or run commands yet. When adding a language or framework, include its dependency manifest and document the exact setup, development, build, and test commands here. Prefer reproducible commands that work from the repository root, such as `npm run build` or `cargo test`, only when the corresponding project configuration exists.

## Coding Style & Naming Conventions

Follow the formatter and linter configured for the chosen language. Until those tools are added, use consistent indentation within each file, descriptive names, and small modules with clear responsibilities. Use conventional lowercase names for directories; follow the language ecosystem’s standard naming for files and identifiers. Add formatting and linting configuration alongside the first implementation rather than relying on undocumented personal settings.

## Testing Guidelines

No test framework or coverage target is configured. Add tests with the first behavior that needs verification, keep test files grouped under `tests/` or beside their implementation according to the selected framework, and name them after the behavior they cover. Document the command that runs the full suite and any required local setup.

## Commit & Pull Request Guidelines

There is no repository history from which to infer a commit convention. Use concise Conventional Commit subjects, for example `feat: add radial key layout` or `fix: handle empty input`. Keep changes focused. Pull requests should explain the purpose, summarize notable changes, list validation performed, link related issues when applicable, and include screenshots for user-visible interface changes.

## Security & Configuration

Keep credentials, tokens, and machine-specific configuration out of commits. Provide safe example configuration with placeholder values when needed, and document required environment variables without including real secrets.
