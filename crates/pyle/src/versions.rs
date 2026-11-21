//! What the version switcher shows, decided without Leptos: the choices it
//! lists and what it tells a visitor who is not on the newest release. A
//! site that wants its own markup builds on these.

use pyle_core::{Manifest, Snapshot, Version};

/// One choice in the switcher: a snapshot, and whether it is the one being
/// viewed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VersionOption {
    pub snapshot: Snapshot,
    pub selected: bool,
}

impl VersionOption {
    /// What the choice reads as: `dev (main)` or `v0.15.0`.
    pub fn label(&self) -> String {
        match self.snapshot {
            Snapshot::Dev => "dev (main)".to_string(),
            Snapshot::Release(version) => version.dir_name(),
        }
    }
}

/// The switcher's choices for a build viewing `current`: `dev`, then every
/// release newest first.
///
/// The build being viewed is always among them and selected, with or
/// without a manifest, so a release never shows `dev` as the current
/// version while its manifest is loading, missing, or older than the
/// release itself.
pub fn version_options(current: Snapshot, manifest: Option<&Manifest>) -> Vec<VersionOption> {
    let listed = manifest.map_or(&[][..], Manifest::versions);
    let viewed = match current {
        Snapshot::Dev => None,
        Snapshot::Release(version) => Some(version),
    };
    let releases = Manifest::new(listed.iter().copied().chain(viewed));
    std::iter::once(Snapshot::Dev)
        .chain(releases.versions().iter().copied().map(Snapshot::Release))
        .map(|snapshot| VersionOption {
            snapshot,
            selected: snapshot == current,
        })
        .collect()
}

/// What the switcher tells a visitor who is not on the newest release.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Notice {
    /// This is main's build, which no release holds yet. `latest` is the
    /// newest release, when there is one to go to: before the first
    /// release the site's root redirects back to `dev`, so there is no
    /// link to offer.
    Unreleased { latest: Option<Version> },
    /// A release newer than the one being viewed exists.
    Outdated { current: Version, latest: Version },
}

impl Notice {
    /// The release to send the visitor to, if there is one.
    pub fn latest(&self) -> Option<Version> {
        match *self {
            Notice::Unreleased { latest } => latest,
            Notice::Outdated { latest, .. } => Some(latest),
        }
    }
}

/// The notice for a build viewing `current`, if it needs one.
///
/// Main's build always gets one, manifest or not. A release gets one only
/// when the manifest lists a later release: a manifest a CDN still holds
/// from before this release must not send the visitor backwards.
pub fn notice(current: Snapshot, manifest: Option<&Manifest>) -> Option<Notice> {
    let latest = manifest.and_then(Manifest::latest);
    match current {
        Snapshot::Dev => Some(Notice::Unreleased { latest }),
        Snapshot::Release(current) => latest
            .filter(|latest| *latest > current)
            .map(|latest| Notice::Outdated { current, latest }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const V13: Version = Version::new(0, 13, 0);
    const V14: Version = Version::new(0, 14, 0);
    const V15: Version = Version::new(0, 15, 0);

    fn snapshots(options: &[VersionOption]) -> Vec<Snapshot> {
        options.iter().map(|o| o.snapshot).collect()
    }

    fn selected(options: &[VersionOption]) -> Vec<Snapshot> {
        options
            .iter()
            .filter(|o| o.selected)
            .map(|o| o.snapshot)
            .collect()
    }

    #[test]
    fn the_choices_are_dev_then_the_releases_newest_first() {
        let manifest = Manifest::new([V14, V15, V13]);
        let options = version_options(Snapshot::Dev, Some(&manifest));
        assert_eq!(
            snapshots(&options),
            [
                Snapshot::Dev,
                Snapshot::Release(V15),
                Snapshot::Release(V14),
                Snapshot::Release(V13)
            ]
        );
    }

    #[test]
    fn main_s_build_selects_dev() {
        let manifest = Manifest::new([V14]);
        let options = version_options(Snapshot::Dev, Some(&manifest));
        assert_eq!(selected(&options), [Snapshot::Dev]);
    }

    #[test]
    fn a_release_without_a_manifest_still_selects_itself() {
        let options = version_options(Snapshot::Release(V14), None);
        assert_eq!(
            (snapshots(&options), selected(&options)),
            (
                vec![Snapshot::Dev, Snapshot::Release(V14)],
                vec![Snapshot::Release(V14)]
            )
        );
    }

    #[test]
    fn a_release_the_manifest_omits_takes_its_place_in_the_order() {
        let manifest = Manifest::new([V15, V13]);
        let options = version_options(Snapshot::Release(V14), Some(&manifest));
        assert_eq!(
            (snapshots(&options), selected(&options)),
            (
                vec![
                    Snapshot::Dev,
                    Snapshot::Release(V15),
                    Snapshot::Release(V14),
                    Snapshot::Release(V13)
                ],
                vec![Snapshot::Release(V14)]
            )
        );
    }

    #[test]
    fn a_release_the_manifest_lists_appears_once() {
        let manifest = Manifest::new([V15, V14]);
        let options = version_options(Snapshot::Release(V14), Some(&manifest));
        assert_eq!(
            snapshots(&options),
            [
                Snapshot::Dev,
                Snapshot::Release(V15),
                Snapshot::Release(V14)
            ]
        );
    }

    #[test]
    fn the_choices_read_as_dev_and_v_numbers() {
        let labels: Vec<String> = version_options(Snapshot::Release(V14), None)
            .iter()
            .map(VersionOption::label)
            .collect();
        assert_eq!(labels, ["dev (main)", "v0.14.0"]);
    }

    #[test]
    fn main_s_build_is_unreleased_and_points_at_the_newest_release() {
        let manifest = Manifest::new([V13, V14]);
        assert_eq!(
            notice(Snapshot::Dev, Some(&manifest)),
            Some(Notice::Unreleased { latest: Some(V14) })
        );
    }

    #[test]
    fn main_s_build_before_the_first_release_has_nowhere_to_point() {
        assert_eq!(
            notice(Snapshot::Dev, Some(&Manifest::default())),
            Some(Notice::Unreleased { latest: None })
        );
    }

    #[test]
    fn main_s_build_is_unreleased_without_a_manifest_too() {
        assert_eq!(
            notice(Snapshot::Dev, None),
            Some(Notice::Unreleased { latest: None })
        );
    }

    #[test]
    fn an_older_release_is_outdated() {
        let manifest = Manifest::new([V13, V14]);
        assert_eq!(
            notice(Snapshot::Release(V13), Some(&manifest)),
            Some(Notice::Outdated {
                current: V13,
                latest: V14
            })
        );
    }

    #[test]
    fn the_newest_release_needs_no_notice() {
        let manifest = Manifest::new([V13, V14]);
        assert_eq!(notice(Snapshot::Release(V14), Some(&manifest)), None);
    }

    #[test]
    fn a_release_newer_than_the_manifest_s_latest_is_not_sent_back() {
        let manifest = Manifest::new([V13]);
        assert_eq!(notice(Snapshot::Release(V14), Some(&manifest)), None);
    }

    #[test]
    fn a_release_without_a_manifest_needs_no_notice() {
        assert_eq!(notice(Snapshot::Release(V14), None), None);
    }

    #[test]
    fn releases_compare_by_number_not_by_text() {
        let manifest = Manifest::new([Version::new(0, 10, 0)]);
        assert_eq!(
            notice(Snapshot::Release(Version::new(0, 9, 0)), Some(&manifest)),
            Some(Notice::Outdated {
                current: Version::new(0, 9, 0),
                latest: Version::new(0, 10, 0)
            })
        );
    }

    #[test]
    fn each_notice_names_the_release_to_go_to() {
        assert_eq!(
            [
                Notice::Unreleased { latest: Some(V14) }.latest(),
                Notice::Unreleased { latest: None }.latest(),
                Notice::Outdated {
                    current: V13,
                    latest: V14
                }
                .latest()
            ],
            [Some(V14), None, Some(V14)]
        );
    }
}
