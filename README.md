# pyle

pyle holds the Leptos components my project landing pages share, such as
the version switcher, code tabs, and roadmap, so a fix lands once for
every site.

**Status:** the `pyle` binary that deploys a site's versions is in place,
and the first component, the version switcher, comes next. It is not on
crates.io.

## What goes here

The sites that use pyle each started as a copy of another, and the copies
drifted: the version switcher's fix for sending a visitor back to an older
release had to land three times. pyle takes the parts every site shares
(the version switcher, code blocks and tabs, a data-driven roadmap, the
feature card, the footer, and the banner), and each site themes them
through CSS variables. The hero, the feature list, and anything else that
is a site's own content stay in the site.

Each component is built with a [theoria](https://github.com/padamson/theoria)
story, for viewing it in the gallery, and
[leptotes](https://github.com/padamson/leptotes) tests, which render it to
HTML in a plain `cargo test`.

## Installation

Take it from git, pinned to a commit:

```toml
[dependencies]
pyle = { git = "https://github.com/padamson/pyle", rev = "<sha>" }
```

## Deploying a versioned site

The `pyle` binary (the `pyle-cli` crate) puts a build into a static site,
one folder per version. A release goes in `v<version>/` and main's build
in `dev/`. `versions.json` at the root lists the releases newest first,
and the root `index.html` redirects to the newest (or to `dev/` before
the first release). It works on a directory, so the workflow checks out
the branch that serves the site and pushes it afterwards.

```bash
cargo install --git https://github.com/padamson/pyle --rev <sha> --locked pyle-cli
pyle add dist --as 0.15.0 --site gh --prefix /cqa-grader --title cqa-grader
```

`--as dev` deploys main's build. `--prefix` is the path the site is
served under: `/<repo>` for GitHub project Pages, `/` for a custom
domain. `pyle manifest`, with the same `--site`, `--prefix`, and
`--title`, rewrites `versions.json` and the redirect without adding
anything. A `CNAME`, `.nojekyll`, or any other file at the root is the
workflow's to write.

The build must have an `index.html` at its top, since that is the page
the snapshot's URL serves. Adding a snapshot that is already in the site
replaces it, and `pyle` prints `replaced` for it where a new one prints
`copied`. The old snapshot stays in place until the new copy is whole.
`pyle manifest` fails on a site with no release and no `dev/`, since the
redirect would have nowhere to go.

## Development

See [CLAUDE.md](CLAUDE.md) for development commands.

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (MSRV: 1.95)
- [prek](https://github.com/j178/prek) for pre-commit hooks: `cargo install prek && prek install --overwrite`

### Build and test

```bash
cargo build
cargo nextest run
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <http://opensource.org/licenses/MIT>)

at your option.
