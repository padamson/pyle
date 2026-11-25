# pyle usage

Each component gets an entry here when it lands: its signature, one line
of intent, and the mistake a first use makes.

## Styles and theme

- `<PyleStyles theme=Theme />`: puts pyle's stylesheet in the page.
  Mount it once, above the first component. `theme` is optional; the
  default is a plain look that follows the system's light or dark. It
  also takes a `Signal<Theme>`, for a page that changes its look while
  open: a change writes the stylesheet again and redraws nothing else.
- `Theme { palette, dark, radius, font }` and `Palette { page, surface,
  surface_strong, border, text, text_muted, accent, accent_hover }`: each
  field is a `&'static str` of CSS, written to `:root` as `--pyle-*`.
  `dark: None` is a site that looks the same either way. To set one
  variable from CSS instead, write it outside any layer (or in a layer
  ranked above `pyle`); in `@layer base` or Tailwind's `@theme` it loses
  to the theme's value.
- First-use mistake: leaving the layer order out of the site's stylesheet.
  pyle's rules sit in `@layer pyle`, which is declared when `PyleStyles`
  runs, after the site's sheet, so it outranks the site's utilities unless
  the sheet names the order first. Tailwind v4: `@layer theme, base,
  components, pyle, utilities;` above `@import "tailwindcss";`. Hand-written
  layers: `@layer base, pyle, utilities;`.

## Page and sections

- `<LandingPage>{sections}</LandingPage>`: the page, in the theme's page
  color, text color, and font, with the sections stacked top to bottom in
  the order given. Optional: `class`. It lays out nothing inside a
  section; each section sets its own width. On the `<body>` that holds it,
  it zeroes the margin and sets the page's color, and with a `dark` palette
  it sets `color-scheme: light dark` on the root.
- `<Hero title tagline>{links}</Hero>`: the opening section. `tagline` and
  the children (links or buttons, in a row under the tagline) are
  optional. Also takes `class`.
- A page is pyle's sections and nothing else. A section the site needs and
  pyle lacks is built in pyle first, with stories and tests.
- First-use mistake: wrapping sections in the site's own `<div>`s for
  spacing or width. Each section owns its spacing; a site that needs
  different spacing asks for a setting on the component.

## Designing a theme

- By hand: in a clone of pyle, `cargo theoria serve`, then the story
  "Theme/Playground" at http://localhost:8081. One control per field (a
  color picker per palette field, the radius in rem, a font from a short
  list); the block above the page is the `Theme` as Rust.
- As an agent: write the `Theme` in the site, then check it where the site
  shows it, in the site's theoria gallery (below) or the built site. The
  fields and what draws with each: `page` behind the whole page, with the
  hero's title (`text`), tagline (`text_muted`), and links (`accent`) on
  it; `surface` behind a bar, `surface_strong` behind a control on it,
  `border` around both, `text` and `text_muted` (labels) on `surface`,
  `accent` for a notice and its link, `accent_hover` for a link under the
  pointer. Check the contrast of `text`, `text_muted`, and `accent` against
  both `page` and `surface`, and of `text` against `surface_strong`.
  Nothing in pyle checks it.
- `StorySite { prefix, theme, ..StorySite::default() }` and
  `pyle::stories_for(site) -> theoria::Stories` (the
  `stories` feature): every component in each state, under the site's
  theme, for the site's own gallery. Pass the same `Theme` the site hands
  to `PyleStyles`, so the gallery cannot drift from the site.
- First-use mistake: copying the site's palette into a second `Theme` for
  the gallery. Make it one `const` and use it in both.

## Version switcher

- `<VersionSwitcher current=Snapshot prefix=Prefix />`: the version bar,
  with the site's `versions.json` fetched from under `prefix`. Optional:
  `dev_notice` (what it says on main's build) and `class`.
- `<VersionBar current prefix manifest=Signal<Option<Manifest>> />`: the
  same bar with the manifest passed in, for a story, a test, or a site that
  reads the manifest itself. Also takes `on_select`, called with the
  picked `Snapshot` in place of opening it.
- `version_options(current, manifest)` and `notice(current, manifest)`:
  what the bar lists and says, as plain values, for a site that wants its
  own markup.
- `current` is the build's `SITE_VERSION` parsed as a `Snapshot`: `dev`, or
  a version with no `v`.
- The bar keeps the ids `version-switcher` and `version-select`, so a page
  holds one.
- A page the browser restores from its back/forward cache shows its own
  build in the menu again, so the site needs no `pageshow` handler.
- First-use mistake: mounting the bar without `PyleStyles`, which leaves it
  unstyled.
