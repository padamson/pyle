# Changelog

All notable changes to this project are documented here.
The format is based on [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased]

### Added
- Repository created from rust-project-template, with its CI, cargo-deny, cargo-vet, and mutation-testing setup. The crate is `publish = false`: sites take it through git, pinned to a commit
- Leptos 0.8 with `csr`, `ssr`, and `hydrate` passed through for the site to pick; theoria behind a `stories` feature and leptotes as a dev-dependency, both from git `main`; a `Build for wasm32` CI job, required, that builds the library under `csr`, with and without `stories`
- MSRV 1.95, theoria's; the MSRV job checks with `--all-features` so the `stories` feature is covered
- A workspace: the components move to `crates/pyle`, and `crates/pyle-core` holds the version model the switcher and the manifest tool will share. Versions parse as exactly three numbers with no leading zeros and order by number; a manifest lists releases newest first and is written in the `versions.json` shape deployed snapshots already read (`latest` is `""` before the first release)
- `pyle-cli`, a binary named `pyle` that replaces each site's `update-manifest.sh` and the version check in its `pages.yml`. `pyle add <build> --as <dev|X.Y.Z>` copies a build into the site's `dev/` or `v<version>/` folder, and both it and `pyle manifest` write `versions.json` and a root `index.html` that redirects to the newest release (or `dev/` before the first). A snapshot already in the site is replaced once the new copy is whole, and the output says `replaced`. `--prefix` must be a URL path, a build with no `index.html` or one that overlaps the site is refused, `pyle manifest` fails on a site with nothing deployed, and `--version` names the commit the binary was built from

[Unreleased]: https://github.com/padamson/pyle/commits/main
