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
- The version switcher, the first component. `VersionSwitcher` fetches the site's `versions.json` and draws `VersionBar`: a menu of `dev` and every release, newest first, with the build being viewed always listed and selected, a notice on main's build, and a link to the newest release from an older one. A page restored from the browser's back/forward cache shows its own build in the menu again, where the sites' copies kept the version the visitor had gone to. A release missing from the manifest is listed in version order, and main's build shows its notice whether or not the manifest can be read. `version_options` and `notice` make the same decisions without Leptos, for a site that wants its own markup
- `Theme` and `PyleStyles`: pyle's own stylesheet, with no Tailwind, as `pyle-*` classes written against `--pyle-*` variables, all in `@layer pyle`. A site names the layer order in its own stylesheet so that its utilities outrank pyle and pyle outranks its reset (README, "Using the version switcher")
- A theoria gallery (`theoria.toml`, run with `./scripts/gallery.sh`) showing the bar under the default theme, with a theme playground: one control per `Theme` field, and the theme written out as the Rust a site pastes in. `pyle::stories_for` (the `stories` feature) gives a site's own gallery the same stories under its theme
- `crates/pyle-e2e`, browser tests that drive it with playwright-rs: the layer order with and without the site's line, the bar reading a manifest that `pyle-cli` wrote, and picking a version. A new `E2E` CI job runs them

[Unreleased]: https://github.com/padamson/pyle/commits/main
