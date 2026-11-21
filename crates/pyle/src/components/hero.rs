//! The opening section of a landing page: the project's name, what it does
//! in a line, and where to go next.

use leptos::prelude::*;

use super::classes;

/// The hero's rules, which [`crate::stylesheet`] puts in `@layer pyle`.
pub(crate) const CSS: &str = include_str!("hero.css");

/// The opening section: `title` as the page's heading, `tagline` under it,
/// and the children, such as links to the source and the docs, under that.
#[component]
pub fn Hero(
    /// The page's heading, usually the project's name.
    #[prop(into)]
    title: String,
    /// What the project does, in a line.
    #[prop(optional, into)]
    tagline: Option<String>,
    /// Where to go next: links or buttons, in a row.
    #[prop(optional)]
    children: Option<Children>,
    /// Classes for the section, after pyle's own.
    #[prop(optional, into)]
    class: String,
) -> impl IntoView {
    view! {
        <section class=classes("pyle-hero", &class)>
            <div class="pyle-hero-inner">
                <h1 class="pyle-hero-title">{title}</h1>
                {tagline.map(|tagline| view! { <p class="pyle-hero-tagline">{tagline}</p> })}
                {children.map(|children| view! { <div class="pyle-hero-actions">{children()}</div> })}
            </div>
        </section>
    }
}

#[cfg(test)]
#[path = "hero.tests.rs"]
mod tests;
