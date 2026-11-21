//! theoria stories for the hero. Compiled only under the `stories` feature.

use leptos::prelude::*;
use theoria::Story;

use super::Hero;
use super::gallery_stories::{StorySite, framed};

/// The hero as `site` shows it, with and without its optional parts.
pub fn for_site(site: StorySite) -> Vec<Story> {
    vec![
        Story::new("Hero/With a tagline and links", move || {
            framed(
                site.theme,
                view! {
                    <Hero title="Project name" tagline="What the project does, in a line or two.">
                        <a href="#source">"Source"</a>
                        <a href="#docs">"Docs"</a>
                    </Hero>
                },
            )
        }),
        Story::new("Hero/Title only", move || {
            framed(site.theme, view! { <Hero title="Project name" /> })
        }),
    ]
}
