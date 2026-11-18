# pyle

pyle holds the Leptos components my project landing pages share, such as
the version switcher, code tabs, and roadmap, so a fix lands once for
every site.

**Status:** empty. The repository has its CI and supply-chain setup, and
the first component (the version switcher, with the binary that writes
its `versions.json`) comes next. It is not on crates.io.

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

## Development

See [CLAUDE.md](CLAUDE.md) for development commands.

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (MSRV: 1.88)
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
