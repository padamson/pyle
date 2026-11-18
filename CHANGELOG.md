# Changelog

All notable changes to this project are documented here.
The format is based on [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased]

### Added
- Repository created from rust-project-template, with its CI, cargo-deny, cargo-vet, and mutation-testing setup. The crate is `publish = false`: sites take it through git, pinned to a commit
- Leptos 0.8 with `csr`, `ssr`, and `hydrate` passed through for the site to pick; theoria behind a `stories` feature and leptotes as a dev-dependency, both from git `main`; a `Build for wasm32` CI job, required, that builds the library under `csr`, with and without `stories`
- MSRV 1.95, theoria's; the MSRV job checks with `--all-features` so the `stories` feature is covered

[Unreleased]: https://github.com/padamson/pyle/commits/main
