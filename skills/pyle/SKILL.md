---
name: pyle
description: Use when a Leptos landing-page site depends on the pyle crate (a `pyle` git dependency in its `Cargo.toml`), when adding or changing a shared landing-page component such as the version switcher, code tabs, or roadmap, or when deciding whether a site's component belongs in pyle or stays in the site.
license: MIT OR Apache-2.0
metadata:
  version: "0.2.0"
  internal: true
---

# pyle

<!--
Hidden from installers (`metadata.internal: true`) until the first
component's API settles; nothing loads it yet. Bump `metadata.version` on
every content change.
-->

## When to use

A Leptos site takes its landing-page components from pyle, or a component
that several sites copy is a candidate to move into it.

## How it works

pyle is where the parts of a project landing page that every site shares
move to, one at a time: the version switcher first, then code blocks and
tabs, the roadmap, the feature card, the footer, and the banner. Each site
themes them through a `Theme`, which sets CSS variables. The hero, the
feature list, and anything else that is the site's own content stay in the
site.

pyle's styles are its own CSS, with no Tailwind, in the cascade layer
`pyle`. A site mounts `<PyleStyles theme=.../>` once and names the layer
order in its own stylesheet, so its utilities outrank pyle and pyle
outranks its reset.

Sites depend on pyle through git, pinned to a commit; it is not on
crates.io.

- [`references/usage.md`](references/usage.md): the API surface.
