//! theoria stories for the landing page frame. Compiled only under the
//! `stories` feature.

use theoria::Story;

use super::gallery_stories::{StorySite, sample_page};

/// A whole page as `site` shows it.
pub fn for_site(site: StorySite) -> Vec<Story> {
    vec![Story::new(
        "Landing page/A page from pyle's sections",
        move || sample_page(site),
    )]
}
