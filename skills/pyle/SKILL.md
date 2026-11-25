---
name: pyle
description: Use when a Leptos landing-page site depends on the pyle crate (a `pyle` git dependency in its `Cargo.toml`), when adding or changing a landing-page component such as the version switcher, a hero, code tabs, or the roadmap, or when a site needs a section pyle does not have yet.
license: MIT OR Apache-2.0
metadata:
  version: "0.4.1"
  internal: true
---

# pyle

<!--
Hidden from installers (`metadata.internal: true`) until the first
component's API settles; nothing loads it yet. Bump `metadata.version` on
every content change.
-->

## When to use

A Leptos site builds its landing page from pyle, or needs a section that
pyle does not have yet.

## How it works

A site's landing page is made entirely of pyle components: section
components (the version switcher, a hero, a feature grid, code blocks and
tabs, the roadmap, the footer, the banner) inside a page component that
frames them. The site supplies its content, a `Theme` (which sets CSS
variables), the order of its sections, and settings on each component,
and composes them. A section a site needs and pyle lacks is built in pyle
first, even for one site; the site does not write its own markup or CSS
for it. The page frame (`LandingPage`), the version switcher, and the hero
are in pyle so far; the rest arrive one at a time.

pyle's styles are its own CSS, with no Tailwind, in the cascade layer
`pyle`. A site mounts `<PyleStyles theme=.../>` once. While a site still
has CSS of its own (during its move to pyle), it names the layer order in
its stylesheet, so its utilities outrank pyle and pyle outranks its
reset.

Sites depend on pyle through git, pinned to a commit; it is not on
crates.io.

- [`references/usage.md`](references/usage.md): the API surface.
