//! The bar across the top of a versioned site: which snapshot this is, a
//! menu of the others, and a notice when the visitor is not on the newest
//! release.

use leptos::prelude::*;
use leptos::task::spawn_local;
use pyle_core::{Manifest, Prefix, Snapshot};

use super::classes;
use crate::versions::{Notice, notice, version_options};

/// The bar's rules, which [`crate::stylesheet`] puts in `@layer pyle`.
pub(crate) const CSS: &str = include_str!("version_bar.css");

/// What the bar says on main's build unless the site says otherwise.
pub const DEFAULT_DEV_NOTICE: &str = "Unreleased build of main.";

/// The version bar, drawn from a manifest the caller holds.
///
/// [`VersionSwitcher`] is this bar with the manifest fetched for it, and is
/// what a site mounts. Use `VersionBar` where the manifest comes from
/// somewhere else: a story, a test, or a site that reads it itself.
///
/// The bar keeps the ids `version-switcher` and `version-select`, so one
/// page holds one bar.
#[component]
pub fn VersionBar(
    /// The snapshot this build is.
    current: Snapshot,
    /// The path the site is served under.
    prefix: Prefix,
    /// The site's manifest, or `None` until it is read.
    #[prop(into)]
    manifest: Signal<Option<Manifest>>,
    /// What the bar says on main's build.
    #[prop(into, default = DEFAULT_DEV_NOTICE.to_string())]
    dev_notice: String,
    /// Called with the snapshot a visitor picks, in place of opening it.
    #[prop(optional, into)]
    on_select: Option<Callback<Snapshot>>,
    /// Classes for the bar, after pyle's own.
    #[prop(optional, into)]
    class: String,
) -> impl IntoView {
    let options = move || manifest.with(|m| version_options(current, m.as_ref()));
    let note = move || manifest.with(|m| notice(current, m.as_ref()));
    let menu = NodeRef::<leptos::html::Select>::new();
    // A page restored from the back/forward cache comes back as the visitor
    // left it, with the version they went to still picked, and picking it
    // again would change nothing. An effect, so the listener is added in
    // the browser and never while a server or a test renders the bar.
    Effect::new(move |_| {
        let listener = window_event_listener(leptos::ev::pageshow, move |_| {
            if let Some(menu) = menu.get_untracked() {
                menu.set_value(&current.to_string());
            }
        });
        on_cleanup(move || listener.remove());
    });
    let select = {
        let prefix = prefix.clone();
        move |ev| {
            // The values are the ones this bar wrote, so anything else is
            // not a choice.
            let Ok(snapshot) = event_target_value(&ev).parse::<Snapshot>() else {
                return;
            };
            match on_select {
                Some(on_select) => on_select.run(snapshot),
                None => {
                    let _ = window()
                        .location()
                        .set_href(&prefix.snapshot_url(&snapshot));
                }
            }
        }
    };

    view! {
        <div id="version-switcher" class=classes("pyle-version-bar", &class)>
            <div class="pyle-version-bar-inner">
                <label for="version-select" class="pyle-version-label">
                    "Version"
                </label>
                <select
                    id="version-select"
                    class="pyle-version-select"
                    node_ref=menu
                    on:change=select
                >
                    {move || {
                        options()
                            .into_iter()
                            .map(|option| {
                                view! {
                                    <option
                                        value=option.snapshot.to_string()
                                        selected=option.selected
                                    >
                                        {option.label()}
                                    </option>
                                }
                            })
                            .collect_view()
                    }}
                </select>
                {move || {
                    note()
                        .map(|note| {
                            let link = note
                                .latest()
                                .map(|latest| {
                                    let href = prefix.snapshot_url(&Snapshot::Release(latest));
                                    view! {
                                        <a href=href class="pyle-version-link">
                                            {format!("Go to {} →", latest.dir_name())}
                                        </a>
                                    }
                                });
                            view! {
                                <span class="pyle-version-notice">
                                    <span>{message(&note, &dev_notice)}</span>
                                    {link}
                                </span>
                            }
                        })
                }}
            </div>
        </div>
    }
}

/// The version bar for a site: [`VersionBar`], with the manifest fetched
/// from under `prefix` once the page is in the browser.
///
/// Until the manifest arrives, or when it cannot be read (a local preview,
/// a site with no deploy yet), the bar lists `dev` and the build being
/// viewed.
#[component]
pub fn VersionSwitcher(
    /// The snapshot this build is.
    current: Snapshot,
    /// The path the site is served under.
    prefix: Prefix,
    /// What the bar says on main's build.
    #[prop(into, default = DEFAULT_DEV_NOTICE.to_string())]
    dev_notice: String,
    /// Classes for the bar, after pyle's own.
    #[prop(optional, into)]
    class: String,
) -> impl IntoView {
    let manifest = RwSignal::new(None::<Manifest>);
    let url = prefix.manifest_url();
    // An effect, so the fetch runs in the browser and never while a server
    // or a test renders the bar.
    Effect::new(move |_| {
        let url = url.clone();
        spawn_local(async move {
            if let Some(fetched) = fetch_manifest(&url).await {
                manifest.set(Some(fetched));
            }
        });
    });
    view! { <VersionBar current prefix manifest dev_notice class /> }
}

async fn fetch_manifest(url: &str) -> Option<Manifest> {
    let response = gloo_net::http::Request::get(url).send().await.ok()?;
    if !response.ok() {
        return None;
    }
    response.json().await.ok()
}

fn message(note: &Notice, dev_notice: &str) -> String {
    match note {
        Notice::Unreleased { .. } => dev_notice.to_string(),
        Notice::Outdated { current, .. } => format!(
            "You're viewing {}. A newer release is available.",
            current.dir_name()
        ),
    }
}

/// The bar, for the tests and the stories' tests to find it by.
#[cfg(test)]
pub(crate) const BAR: &str = "#version-switcher";

/// The notice's words, without its link.
#[cfg(test)]
pub(crate) const NOTICE_WORDS: &str = ".pyle-version-notice > span";

#[cfg(test)]
#[path = "version_bar.tests.rs"]
mod tests;
