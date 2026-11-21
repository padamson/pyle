# pyle

pyle holds the Leptos components my project landing pages are built from,
so a fix lands once for every site. A site's landing page is made of pyle
components and nothing else.

**Status:** the `pyle` binary that deploys a site's versions and the
first component, the version switcher, are in place. It is not on
crates.io.

## What goes here

The sites that use pyle each started as a copy of another, and the copies
drifted: the version switcher's fix for sending a visitor back to an older
release had to land three times. So the whole landing page comes from
pyle: section components (the version switcher, a hero, a feature grid,
code blocks and tabs, a data-driven roadmap, the footer, and the banner),
and a page component that frames them. A site supplies its content, its
`Theme`, and the order of its sections, and puts them together by
composition. When a site needs a section pyle does not have, the section
is built in pyle, even if only one site uses it.

Sites still look different. Each picks its own sections and their order,
its own theme, and its own settings on each component: square or rounded
corners, shadows or none, motion or none.

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

## Using the version switcher

Mount `PyleStyles` once with the site's theme, then the switcher, with
the snapshot this build is and the path the site is served under:

```rust
use leptos::prelude::*;
use pyle::{Prefix, PyleStyles, Snapshot, Theme, VersionSwitcher};

#[component]
fn App() -> impl IntoView {
    let current: Snapshot = env!("SITE_VERSION").parse().expect("dev or X.Y.Z");
    view! {
        <PyleStyles theme=Theme::default() />
        <VersionSwitcher current prefix=Prefix::new("/cqa-grader") />
    }
}
```

The switcher reads `versions.json` from under the prefix and lists `dev`
and every release, newest first. On main's build it says so (`dev_notice`
sets the words), and on an older release it links to the newest.
`VersionBar` is the same bar with the manifest passed in.

pyle's styles are its own CSS: classes named `pyle-*`, written against
`--pyle-*` variables that `Theme` sets on `:root`, all in the cascade
layer `pyle`. A site can set any of the variables in its own CSS, outside
any layer or in a layer it ranks above `pyle` (one set in a lower layer,
such as `base`, loses to the theme's), and can pass classes to a
component's `class`. For those classes to win, the
site's stylesheet has to rank the layer below its utilities, and above
its reset, before pyle's styles arrive. In a Tailwind v4 `input.css`
that is one line above the import:

```css
@layer theme, base, components, pyle, utilities;
@import "tailwindcss";
```

A site with its own `base` and `utilities` layers writes `@layer base,
pyle, utilities;`. Without the line, pyle's layer is declared last and
outranks the utilities.

## Choosing a theme

A site's look is a `Theme`: a palette, an optional second palette for
visitors whose system asks for dark, a corner radius, and a font stack.
Each field is CSS, so `"#f2d16b"` and `"rgb(36 56 50 / 0.8)"` both work.

To design one, run pyle's gallery (`./scripts/gallery.sh serve` in a clone, under
Development below) and open "Theme/Playground". Each control is one
field. The components redraw as you type, and the block under them is the
`Theme` as Rust, to paste into the site.

A site with its own theoria gallery shows pyle's components there, under
the theme it mounts. Turn on pyle's `stories` feature where the site's
stories are built, and return this from the gallery's aggregator:

```rust
use pyle::{StorySite, Theme};

pub fn all_stories() -> theoria::Stories {
    pyle::stories_for(StorySite {
        prefix: "/cqa-grader",
        page: "#1b2a24",
        theme: Theme::default(), // the `Theme` the site hands to `PyleStyles`
        ..StorySite::default()
    })
}
```

`page` is the color of the page behind the components. pyle's own gallery
holds no site's theme, so a site that changes its look changes one place.

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
./scripts/gallery.sh serve   # the component gallery, http://localhost:8081
```

The script runs the [theoria](https://github.com/padamson/theoria) CLI on
the dependencies `Cargo.lock` pins; the gallery also needs the `wasm32-unknown-unknown` target and
[Trunk](https://trunkrs.dev). The browser tests in `crates/pyle-e2e`
drive that gallery in Chromium; [CLAUDE.md](CLAUDE.md) has the commands.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <http://opensource.org/licenses/MIT>)

at your option.
