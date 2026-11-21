//! theoria stories for the version bar. Compiled only under the `stories`
//! feature. pyle's gallery shows the bar under the default theme. A site's
//! look is the site's, so its own gallery shows the same states under its
//! theme, through [`crate::stories_for`].

use leptos::prelude::*;
use pyle_core::{Manifest, Prefix, Snapshot, Version};
use theoria::{Story, story};

use super::gallery_stories::{StorySite, framed};
use super::{DEFAULT_DEV_NOTICE, VersionBar, VersionSwitcher};
use crate::{PyleStyles, notice};

const V13: Version = Version::new(0, 13, 0);
const V14: Version = Version::new(0, 14, 0);
const V15: Version = Version::new(0, 15, 0);

/// What a visitor can arrive at.
struct State {
    /// The story's name.
    name: &'static str,
    /// The build the visitor is viewing.
    current: Snapshot,
    /// The manifest the bar has read by then.
    manifest: Option<Manifest>,
    /// What the story says under the bar, for a state the bar alone does
    /// not explain.
    caption: Option<&'static str>,
}

fn states() -> [State; 5] {
    let releases = || Some(Manifest::new([V13, V14, V15]));
    [
        State {
            name: "Version bar/On main, with a release to go to",
            current: Snapshot::Dev,
            manifest: releases(),
            caption: None,
        },
        State {
            name: "Version bar/On main, before the first release",
            current: Snapshot::Dev,
            manifest: Some(Manifest::default()),
            caption: None,
        },
        State {
            name: "Version bar/On an older release",
            current: Snapshot::Release(V13),
            manifest: releases(),
            caption: None,
        },
        State {
            name: "Version bar/On the newest release",
            current: Snapshot::Release(V15),
            manifest: releases(),
            caption: None,
        },
        State {
            name: "Version bar/Without the manifest",
            current: Snapshot::Release(V14),
            manifest: None,
            caption: Some(
                "The bar on v0.14.0 before the site's versions.json has loaded, and \
                 for good when it cannot be: a local preview, a site with nothing \
                 deployed, a failed fetch. It knows only main's build and this one, \
                 so it lists those two, and it shows no notice, since it cannot tell \
                 whether a newer release exists.",
            ),
        },
    ]
}

/// The bar as `site` shows it, over the site's page color, starting on the
/// build `current`.
///
/// On a site, picking a version or following the notice's link opens
/// another page, built from another snapshot, with its own bar. A story
/// has no other page, so it draws that page's bar in place and says where
/// the visitor landed.
fn on_site(
    site: StorySite,
    current: Snapshot,
    manifest: Option<Manifest>,
    dev_notice: String,
    class: String,
    caption: Option<&'static str>,
) -> impl IntoView {
    let prefix = Prefix::new(site.prefix);
    // The build whose bar is on show: the story's, until a pick or the link.
    let shown = RwSignal::new(current);
    let moved = move || shown.get() != current;
    let landed = {
        let prefix = prefix.clone();
        move || {
            moved().then(|| {
                format!(
                    "A visitor would land on {}. This is the bar on that page.",
                    prefix.snapshot_url(&shown.get())
                )
            })
        }
    };
    let follow_link = {
        let manifest = manifest.clone();
        move |ev: leptos::ev::MouseEvent| {
            let on_link = event_target::<leptos::web_sys::Element>(&ev)
                .closest("a")
                .is_ok_and(|link| link.is_some());
            if !on_link {
                return;
            }
            // The link's address is not served by the gallery.
            ev.prevent_default();
            let newest =
                notice(shown.get_untracked(), manifest.as_ref()).and_then(|notice| notice.latest());
            if let Some(newest) = newest {
                shown.set(Snapshot::Release(newest));
            }
        }
    };
    framed(
        site.theme,
        site.page,
        view! {
            <div on:click=follow_link>
                {move || {
                    view! {
                        <VersionBar
                            current=shown.get()
                            prefix=prefix.clone()
                            manifest=Signal::stored(manifest.clone())
                            dev_notice=dev_notice.clone()
                            on_select=Callback::new(move |snapshot| shown.set(snapshot))
                            class=class.clone()
                        />
                    }
                }}
            </div>
            <p style="padding: 1rem 1.5rem; font: 0.875rem ui-monospace, monospace; color: GrayText">
                <span class="story-lands">{landed}</span>
                {move || (!moved()).then_some(caption).flatten()}
                <Show when=moved>
                    " "
                    <button on:click=move |_| shown.set(current)>"Back to the story's build"</button>
                </Show>
            </p>
        },
    )
}

/// The bar with every input editable:
///
/// - `current`: the build being viewed, `dev` or a version such as `0.14.0`.
/// - `newest`: the newest release the manifest lists, or empty for a site
///   with no release yet.
/// - `dev_notice`: what the bar says on main's build. It is drawn only
///   while `current` is `dev`.
/// - `class`: classes from the site.
///
/// It starts on `dev`, where all four show. Editing a control starts the
/// story again from `current`, so a version picked in the bar's own menu
/// is forgotten.
#[story(
    name = "Version bar/Playground",
    current = "dev",
    newest = "0.15.0",
    dev_notice = DEFAULT_DEV_NOTICE,
    class = ""
)]
fn playground(current: String, newest: String, dev_notice: String, class: String) -> impl IntoView {
    let site = StorySite::default();
    match current.parse::<Snapshot>() {
        Ok(current) => {
            let manifest = Manifest::new(newest.parse::<Version>().ok());
            on_site(site, current, Some(manifest), dev_notice, class, None).into_any()
        }
        Err(err) => view! { <p>{err.to_string()}</p> }.into_any(),
    }
}

/// The line a Tailwind site puts above `@import "tailwindcss"`: pyle's
/// layer over the reset, under the utilities.
const TAILWIND_ORDER: &str = "@layer theme, base, components, pyle, utilities;";

/// The same line for a site that writes its own `base` and `utilities`
/// layers, as asbuilt does.
const PLAIN_ORDER: &str = "@layer base, pyle, utilities;";

/// What every layered site stylesheet here holds: the reset that zeroes
/// each element's border and padding, and one utility.
const RESET_AND_UTILITY: &str = "\
@layer base {
  *, ::after, ::before { box-sizing: border-box; border: 0 solid; margin: 0; padding: 0; }
}
@layer utilities {
  .bg-fuchsia-500 { background-color: rgb(255 0 255); }
}
";

/// The layer statements a Tailwind v4 build opens with (`tailwindcss` 4.3,
/// read from a site's built stylesheet).
const TAILWIND_STATEMENTS: &str = "@layer properties;\n@layer theme, base, components, utilities;";

/// A site setting one of pyle's variables in its own CSS, outside any
/// layer, which outranks every layer.
const OWN_SURFACE: &str = ":root { --pyle-surface: rgb(0 128 128); }\n";

/// The bar with the site's `class` on it, after a site stylesheet made of
/// `statements` a line each, the reset and utility above, and `rest`, with
/// `caption` under it saying what the stylesheet holds and what to see. The
/// site's sheet comes first, as it does on a page whose `<head>` links it.
///
/// These stories are fixtures for the browser tests in crates/pyle-e2e,
/// which read the bar's computed styles; the captions are for a person who
/// opens one.
fn layered(
    statements: &'static [&'static str],
    rest: &'static str,
    class: &'static str,
    caption: &'static str,
) -> impl IntoView {
    let statements = statements.join("\n");
    view! {
        <style inner_html=format!("{statements}\n{RESET_AND_UTILITY}{rest}")></style>
        <PyleStyles />
        <VersionBar
            current=Snapshot::Release(V14)
            prefix=Prefix::new("/my-site")
            manifest=Signal::stored(None)
            class
        />
        <p style="max-width: 48rem; padding: 1rem 1.5rem; font: 0.875rem/1.5 ui-sans-serif, system-ui, sans-serif; color: GrayText">
            {caption}
        </p>
    }
}

/// The version bar in each state a visitor can arrive at, as `site` shows
/// it.
pub fn for_site(site: StorySite) -> Vec<Story> {
    states()
        .into_iter()
        .map(|state| {
            Story::new(state.name, move || {
                on_site(
                    site,
                    state.current,
                    state.manifest.clone(),
                    site.dev_notice.to_string(),
                    String::new(),
                    state.caption,
                )
            })
        })
        .collect()
}

/// pyle's own gallery: the states under the default theme, the playground,
/// and the stories the browser tests drive.
pub fn stories() -> Vec<Story> {
    let mut stories = for_site(StorySite::default());
    stories.extend([
        playground(),
        // In the gallery alone there is no manifest under `/my-site`, so the
        // bar stays as it first renders. The browser tests serve one.
        Story::new("Version switcher/Reading the site's manifest", || {
            view! {
                <PyleStyles />
                <VersionSwitcher current=Snapshot::Release(V14) prefix=Prefix::new("/my-site") />
            }
        }),
        Story::new("Layering/A Tailwind site that names the order", || {
            layered(
                &[TAILWIND_ORDER, TAILWIND_STATEMENTS],
                "",
                "bg-fuchsia-500",
                "The site's stylesheet opens with the line the README asks for, \
                 @layer theme, base, components, pyle, utilities; then has a reset \
                 that zeroes every element's border, as Tailwind's does, and one \
                 utility, bg-fuchsia-500, which the bar carries. The bar should be \
                 fuchsia, since the site's utilities outrank pyle, and the menu \
                 should keep its border, since pyle outranks the reset.",
            )
        }),
        Story::new("Layering/A Tailwind site that names no order", || {
            layered(
                &[TAILWIND_STATEMENTS],
                "",
                "bg-fuchsia-500",
                "The same stylesheet without the line. pyle's styles arrive after \
                 the site's, so pyle is the last layer named and outranks the \
                 site's: the bar stays grey and ignores bg-fuchsia-500. This is \
                 what a site that leaves the line out gets.",
            )
        }),
        Story::new(
            "Layering/A site with its own layers that names the order",
            || {
                layered(
                    &[PLAIN_ORDER],
                    "",
                    "bg-fuchsia-500",
                    "A site without Tailwind that writes its own base and utilities \
                     layers, as asbuilt does, and opens with @layer base, pyle, \
                     utilities; The bar should be fuchsia and the menu should keep \
                     its border, as on the Tailwind site.",
                )
            },
        ),
        Story::new("Layering/A site that sets one of pyle's variables", || {
            layered(
                &[TAILWIND_ORDER, TAILWIND_STATEMENTS],
                OWN_SURFACE,
                "",
                "The Tailwind site's stylesheet with the line, plus --pyle-surface \
                 set on :root outside any layer, which outranks every layer. The \
                 bar should be teal: a site can change one of its theme's values \
                 from its own CSS.",
            )
        }),
    ]);
    stories
}
