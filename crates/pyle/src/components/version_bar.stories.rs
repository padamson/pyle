//! theoria stories for the version bar. Compiled only under the `stories`
//! feature. pyle's gallery shows the bar under the default theme. A site's
//! look is the site's, so its own gallery shows the same states under its
//! theme, through [`crate::stories_for`].
//!
//! `VersionSwitcher` has no story of its own. It is this bar with the
//! site's `versions.json` fetched, and a gallery serves none, so its story
//! would be "Without the manifest" again. The browser tests drive it on
//! pyle's deployed site.

use leptos::prelude::*;
use pyle_core::{Manifest, Prefix, Snapshot, Version};
use theoria::{Story, story};

use super::gallery_stories::{StorySite, framed};
use super::{DEFAULT_DEV_NOTICE, VersionBar};
use crate::notice;

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
    /// The story's description, for a state the bar alone does not
    /// explain.
    description: Option<&'static str>,
}

fn states() -> [State; 5] {
    let releases = || Some(Manifest::new([V13, V14, V15]));
    [
        State {
            name: "Version bar/On main, with a release to go to",
            current: Snapshot::Dev,
            manifest: releases(),
            description: None,
        },
        State {
            name: "Version bar/On main, before the first release",
            current: Snapshot::Dev,
            manifest: Some(Manifest::default()),
            description: None,
        },
        State {
            name: "Version bar/On an older release",
            current: Snapshot::Release(V13),
            manifest: releases(),
            description: None,
        },
        State {
            name: "Version bar/On the newest release",
            current: Snapshot::Release(V15),
            manifest: releases(),
            description: None,
        },
        State {
            name: "Version bar/Without the manifest",
            current: Snapshot::Release(V14),
            manifest: None,
            description: Some(
                "The story starts on v0.14.0, as the bar is before the site's \
                 `versions.json` has loaded, and for good when it cannot be: a local \
                 preview, a site with nothing deployed, a failed fetch. The bar knows \
                 only main's build and its own, so it lists those two, and it shows \
                 no notice, since it cannot tell whether a newer release exists.",
            ),
        },
    ]
}

/// The bar of the build `shown` holds, drawn again for another build when
/// a visitor picks one or follows the notice's link.
///
/// On a site, either opens another page, built from another snapshot, with
/// its own bar. A story has no other page, and the gallery does not serve
/// the link's address, so the story draws that page's bar in place.
pub(crate) fn staying_bar(
    shown: RwSignal<Snapshot>,
    prefix: Prefix,
    manifest: Option<Manifest>,
    dev_notice: String,
    class: String,
) -> impl IntoView {
    let follow_link = {
        let manifest = manifest.clone();
        move |ev: leptos::ev::MouseEvent| {
            let on_link = event_target::<leptos::web_sys::Element>(&ev)
                .closest("a")
                .is_ok_and(|link| link.is_some());
            if !on_link {
                return;
            }
            ev.prevent_default();
            let newest =
                notice(shown.get_untracked(), manifest.as_ref()).and_then(|notice| notice.latest());
            if let Some(newest) = newest {
                shown.set(Snapshot::Release(newest));
            }
        }
    };
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
                .into_any()
            }}
        </div>
    }
}

/// The bar as `site` shows it, over the site's page color, starting on the
/// build `current`, with a line under it saying where a visitor who picked
/// a version or followed the link would have landed.
fn on_site(
    site: StorySite,
    current: Snapshot,
    manifest: Option<Manifest>,
    dev_notice: String,
    class: String,
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
    framed(
        site.theme,
        view! {
            {staying_bar(shown, prefix, manifest, dev_notice, class)}
            <p style="padding: 1rem 1.5rem; font: 0.875rem ui-monospace, monospace; color: GrayText">
                <span class="story-lands">{landed}</span>
                <Show when=moved>
                    " "
                    <button on:click=move |_| shown.set(current)>"Back to the story's build"</button>
                </Show>
            </p>
        }
        // Erased, so the frame's view type stays shallow enough for rustc's
        // query depth limit, which a cold build of the stories exceeds.
        .into_any(),
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
            on_site(site, current, Some(manifest), dev_notice, class).into_any()
        }
        Err(err) => view! { <p>{err.to_string()}</p> }.into_any(),
    }
}

/// The version bar in each state a visitor can arrive at, as `site` shows
/// it.
pub fn for_site(site: StorySite) -> Vec<Story> {
    states()
        .into_iter()
        .map(|state| {
            let story = Story::new(state.name, move || {
                on_site(
                    site,
                    state.current,
                    state.manifest.clone(),
                    site.dev_notice.to_string(),
                    String::new(),
                )
            });
            match state.description {
                Some(description) => story.with_description(description),
                None => story,
            }
        })
        .collect()
}

/// pyle's own gallery: the states under the default theme, and the
/// playground.
pub fn stories() -> Vec<Story> {
    let mut stories = for_site(StorySite::default());
    stories.push(playground());
    stories
}
