//! The versions of a static site deployed one folder per release, and the
//! `versions.json` that lists them.
//!
//! A versioned site keeps each release in its own folder (`v0.14.0/`), the
//! build of `main` in `dev/`, and a `versions.json` at its root listing the
//! releases newest first. `pyle-cli` writes that manifest and the version
//! switcher reads it in the browser; both go through this crate, so they
//! agree on what a version is and which one is newest.
//!
//! ```
//! use pyle_core::{Manifest, Prefix, Snapshot, Version};
//!
//! let manifest = Manifest::new(["0.9.0", "0.10.0"].map(|v| v.parse::<Version>().unwrap()));
//! assert_eq!(manifest.latest(), Some(Version::new(0, 10, 0)));
//!
//! let site = Prefix::new("/cqa-grader");
//! let latest = Snapshot::Release(Version::new(0, 10, 0));
//! assert_eq!(site.snapshot_url(&latest), "/cqa-grader/v0.10.0/");
//! ```

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// A release version, `MAJOR.MINOR.PATCH`.
///
/// Versions order by number (`0.10.0` is newer than `0.9.0`), which is the
/// order the switcher and the manifest both use.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version {
    major: u64,
    minor: u64,
    patch: u64,
}

impl Version {
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// The version a release folder holds: `v0.14.0` is 0.14.0. Anything
    /// else (`dev`, `latest`, a folder named without the `v`) is `None`.
    pub fn from_dir_name(name: &str) -> Option<Self> {
        name.strip_prefix('v')?.parse().ok()
    }

    /// The folder a release is deployed to, `v0.14.0`.
    pub fn dir_name(&self) -> String {
        format!("v{self}")
    }
}

/// Parses exactly three numbers with no leading zeros. A pre-release
/// (`0.15.0-rc.1`), a `v` prefix, or `0.15` is an error rather than a
/// guess, since a version that read differently here and in a folder name
/// would send a visitor to a folder that does not exist.
impl FromStr for Version {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('.').map(number);
        match (parts.next(), parts.next(), parts.next(), parts.next()) {
            (Some(Some(major)), Some(Some(minor)), Some(Some(patch)), None) => {
                Ok(Self::new(major, minor, patch))
            }
            _ => Err(ParseError::NotAVersion(s.to_string())),
        }
    }
}

/// One field of a version: ASCII digits, and no leading zero unless the
/// field is `0` itself. An empty field passes both checks and fails the
/// parse.
fn number(field: &str) -> Option<u64> {
    let digits = field.bytes().all(|b| b.is_ascii_digit());
    let canonical = field == "0" || !field.starts_with('0');
    if digits && canonical {
        field.parse().ok()
    } else {
        None
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Which build of the site a page belongs to: the build of `main`, or a
/// release.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Snapshot {
    Dev,
    Release(Version),
}

/// The build of `main`'s name, both as `SITE_VERSION` and as its folder.
const DEV: &str = "dev";

impl Snapshot {
    /// The folder the build is deployed to: `dev` or `v0.14.0`.
    pub fn dir_name(&self) -> String {
        match self {
            Snapshot::Dev => DEV.to_string(),
            Snapshot::Release(version) => version.dir_name(),
        }
    }
}

/// Parses the form a site's build passes in (its `SITE_VERSION`): `dev`, or
/// a bare version such as `0.14.0`.
impl FromStr for Snapshot {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == DEV {
            return Ok(Snapshot::Dev);
        }
        s.parse()
            .map(Snapshot::Release)
            .map_err(|_| ParseError::NotASnapshot(s.to_string()))
    }
}

impl fmt::Display for Snapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Snapshot::Dev => f.write_str(DEV),
            Snapshot::Release(version) => version.fmt(f),
        }
    }
}

/// A string that is not a version, a snapshot, or a prefix. Each variant
/// carries the input, so the caller can say which one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError {
    NotAVersion(String),
    NotASnapshot(String),
    NotAPrefix(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::NotAVersion(s) => write!(
                f,
                "`{s}` is not a version: expected three numbers with no leading zeros, such as 0.14.0"
            ),
            ParseError::NotASnapshot(s) => write!(
                f,
                "`{s}` is not a snapshot: expected `dev` or a version such as 0.14.0"
            ),
            ParseError::NotAPrefix(s) => write!(
                f,
                "`{s}` is not a URL path: expected segments of letters, digits, and - . _ ~ separated by /"
            ),
        }
    }
}

impl std::error::Error for ParseError {}

/// The manifest's file name, at the site root.
pub const MANIFEST_FILE: &str = "versions.json";

/// The path a site is served under: `/cqa-grader` for a GitHub project
/// site, or empty for a site at the root of its domain. Every URL the
/// switcher and the manifest write starts with it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Prefix(String);

impl Prefix {
    /// Normalizes `path` to one leading slash and no trailing one, so
    /// `cqa-grader`, `/cqa-grader`, and `/cqa-grader/` are the same prefix,
    /// and `""` and `/` are both the root. It takes `path` as written, for a
    /// site's own constant; parse a prefix someone typed, which checks it.
    pub fn new(path: &str) -> Self {
        match path.trim_matches('/') {
            "" => Self(String::new()),
            trimmed => Self(format!("/{trimmed}")),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Where the manifest lives: `/cqa-grader/versions.json`.
    pub fn manifest_url(&self) -> String {
        format!("{}/{MANIFEST_FILE}", self.0)
    }

    /// The root of a snapshot: `/cqa-grader/dev/` or `/cqa-grader/v0.14.0/`.
    pub fn snapshot_url(&self, snapshot: &Snapshot) -> String {
        format!("{}/{}/", self.0, snapshot.dir_name())
    }
}

/// Parses a prefix as [`Prefix::new`] does, and refuses one a URL path
/// cannot hold as written (a host, a space, `..`, an empty segment), so a
/// mistyped prefix fails where it was typed rather than in a visitor's
/// browser.
impl FromStr for Prefix {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let prefix = Prefix::new(s);
        let characters = prefix
            .0
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-._~/".contains(&b));
        let segments = prefix
            .0
            .split('/')
            .skip(1)
            .all(|s| !matches!(s, "" | "." | ".."));
        if characters && segments {
            Ok(prefix)
        } else {
            Err(ParseError::NotAPrefix(s.to_string()))
        }
    }
}

/// The published releases, newest first, with no duplicates.
///
/// As `versions.json` it is `{"latest": "0.14.0", "versions": ["0.14.0",
/// "0.13.0"]}`, with `latest` an empty string when nothing is released. That
/// is the shape every snapshot deployed so far reads, so it does not change.
/// Reading is lenient where an old or hand-edited manifest could differ: a
/// `versions` that is missing or not a list is empty, and an entry that is
/// not a version (`"v0.13.0"`, `0.13`, `null`) is dropped, so it never
/// becomes a choice in the switcher and never costs the switcher the rest
/// of the list. `latest` is not read at all; it is always the newest of
/// `versions`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(from = "WireIn")]
pub struct Manifest {
    versions: Vec<Version>,
}

impl Manifest {
    pub fn new(versions: impl IntoIterator<Item = Version>) -> Self {
        let mut versions: Vec<Version> = versions.into_iter().collect();
        versions.sort_unstable_by(|a, b| b.cmp(a));
        versions.dedup();
        Self { versions }
    }

    /// The newest release, or `None` before the first one.
    pub fn latest(&self) -> Option<Version> {
        self.versions.first().copied()
    }

    /// Where the site root sends a visitor: the newest release, or `dev`
    /// before the first one.
    pub fn root_snapshot(&self) -> Snapshot {
        self.latest().map_or(Snapshot::Dev, Snapshot::Release)
    }

    /// Every release, newest first.
    pub fn versions(&self) -> &[Version] {
        &self.versions
    }
}

impl Serialize for Manifest {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        WireOut {
            latest: self.latest().map(|v| v.to_string()).unwrap_or_default(),
            versions: self.versions.iter().map(Version::to_string).collect(),
        }
        .serialize(serializer)
    }
}

#[derive(Serialize)]
struct WireOut {
    latest: String,
    versions: Vec<String>,
}

/// `latest` is left out, so serde skips it whatever it holds.
#[derive(Deserialize)]
struct WireIn {
    versions: Option<List>,
}

/// `versions` as a list, or anything else (`null`, a string, an object),
/// which reads as no releases.
#[derive(Deserialize)]
#[serde(untagged)]
enum List {
    Entries(Vec<Entry>),
    Other(serde::de::IgnoredAny),
}

/// One element of `versions`: a string, or anything else, which is dropped.
#[derive(Deserialize)]
#[serde(untagged)]
enum Entry {
    Text(String),
    Other(serde::de::IgnoredAny),
}

impl From<WireIn> for Manifest {
    fn from(wire: WireIn) -> Self {
        let entries = match wire.versions {
            Some(List::Entries(entries)) => entries,
            Some(List::Other(_)) | None => Vec::new(),
        };
        Manifest::new(entries.iter().filter_map(|entry| match entry {
            Entry::Text(s) => s.parse().ok(),
            Entry::Other(_) => None,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn v(s: &str) -> Version {
        s.parse().unwrap()
    }

    #[test]
    fn versions_order_by_number_not_by_text() {
        assert!(v("0.10.0") > v("0.9.0"));
    }

    #[test]
    fn a_major_outranks_any_minor() {
        assert!(v("1.0.0") > v("0.99.99"));
    }

    #[test]
    fn a_minor_outranks_any_patch() {
        assert!(v("0.2.0") > v("0.1.99"));
    }

    #[test]
    fn a_version_built_from_numbers_is_major_minor_patch() {
        assert_eq!(Version::new(1, 2, 3), v("1.2.3"));
    }

    #[test]
    fn a_version_reads_back_as_it_was_written() {
        for s in ["0.0.0", "0.14.0", "10.20.30"] {
            assert_eq!(v(s).to_string(), s);
        }
    }

    #[test]
    fn anything_but_three_plain_numbers_is_not_a_version() {
        for s in [
            "",
            "0.15",
            "0.15.0.1",
            "v0.15.0",
            "0.15.0-rc.1",
            "+1.0.0",
            "01.0.0",
            "0.01.0",
            " 0.15.0",
            "0..0",
            "garbage",
            "18446744073709551616.0.0",
        ] {
            assert!(
                matches!(s.parse::<Version>(), Err(ParseError::NotAVersion(ref got)) if got == s),
                "{s:?} parsed as {:?}",
                s.parse::<Version>()
            );
        }
    }

    #[test]
    fn a_release_folder_is_the_version_after_a_v() {
        assert_eq!(Version::from_dir_name("v0.14.0"), Some(v("0.14.0")));
    }

    #[test]
    fn a_release_deploys_to_its_version_after_a_v() {
        assert_eq!(v("0.14.0").dir_name(), "v0.14.0");
    }

    #[test]
    fn a_folder_that_is_not_v_then_a_version_is_not_a_release() {
        for name in [
            "0.14.0", "dev", "latest", "vdev", "v", "vv0.14.0", "V0.14.0",
        ] {
            assert_eq!(Version::from_dir_name(name), None, "{name:?}");
        }
    }

    #[test]
    fn dev_and_a_bare_version_are_snapshots() {
        assert_eq!("dev".parse(), Ok(Snapshot::Dev));
        assert_eq!("0.14.0".parse(), Ok(Snapshot::Release(v("0.14.0"))));
    }

    #[test]
    fn a_snapshot_reads_back_as_it_was_written() {
        for s in ["dev", "0.14.0"] {
            assert_eq!(s.parse::<Snapshot>().unwrap().to_string(), s);
        }
    }

    #[test]
    fn anything_but_dev_or_a_bare_version_is_not_a_snapshot() {
        for s in ["v0.14.0", "", "Dev", "main"] {
            assert_eq!(
                s.parse::<Snapshot>(),
                Err(ParseError::NotASnapshot(s.to_string())),
                "{s:?}"
            );
        }
    }

    // `pyle-cli` prints these, so each names the input the user has to fix.
    #[test]
    fn the_version_error_names_the_input() {
        let message = ParseError::NotAVersion("v0.14.0".to_string()).to_string();
        assert!(message.contains("v0.14.0"), "{message}");
    }

    #[test]
    fn the_snapshot_error_names_the_input() {
        let message = ParseError::NotASnapshot("main".to_string()).to_string();
        assert!(message.contains("main"), "{message}");
    }

    #[test]
    fn the_prefix_error_names_the_input() {
        let message = ParseError::NotAPrefix("/my site".to_string()).to_string();
        assert!(message.contains("/my site"), "{message}");
    }

    #[test]
    fn a_parsed_prefix_is_normalized_as_a_new_one() {
        for path in ["cqa-grader", "/my-site_2.0~x/docs/", "", "/"] {
            assert_eq!(path.parse::<Prefix>(), Ok(Prefix::new(path)), "{path:?}");
        }
    }

    #[test]
    fn a_prefix_with_a_character_a_url_path_lacks_is_refused() {
        for bad in [
            "https://padamson.github.io/x",
            "/my site",
            "/a\"b",
            "/caf\u{e9}",
        ] {
            assert_eq!(
                bad.parse::<Prefix>(),
                Err(ParseError::NotAPrefix(bad.to_string())),
                "{bad:?}"
            );
        }
    }

    #[test]
    fn a_prefix_with_an_empty_or_dot_segment_is_refused() {
        for bad in ["/a//b", "/./x", "/../x", "/x/.."] {
            assert_eq!(
                bad.parse::<Prefix>(),
                Err(ParseError::NotAPrefix(bad.to_string())),
                "{bad:?}"
            );
        }
    }

    #[test]
    fn a_prefix_has_one_leading_slash_and_no_trailing_one() {
        for path in [
            "/cqa-grader",
            "cqa-grader",
            "/cqa-grader/",
            "//cqa-grader//",
        ] {
            assert_eq!(Prefix::new(path).as_str(), "/cqa-grader", "{path:?}");
        }
    }

    #[test]
    fn a_prefix_keeps_its_inner_slashes() {
        assert_eq!(Prefix::new("/org/site/").as_str(), "/org/site");
    }

    #[test]
    fn an_empty_prefix_or_a_lone_slash_is_the_root() {
        assert_eq!(Prefix::new(""), Prefix::default());
        assert_eq!(Prefix::new("/"), Prefix::default());
    }

    #[test]
    fn the_manifest_lives_under_the_prefix() {
        assert_eq!(
            Prefix::new("/cqa-grader").manifest_url(),
            "/cqa-grader/versions.json"
        );
    }

    #[test]
    fn each_snapshot_lives_in_its_folder_under_the_prefix() {
        let site = Prefix::new("/cqa-grader");
        for (snapshot, url) in [
            (Snapshot::Dev, "/cqa-grader/dev/"),
            (Snapshot::Release(v("0.1.0")), "/cqa-grader/v0.1.0/"),
        ] {
            assert_eq!(site.snapshot_url(&snapshot), url);
        }
    }

    #[test]
    fn a_root_site_s_manifest_is_at_the_domain_root() {
        assert_eq!(Prefix::default().manifest_url(), "/versions.json");
    }

    #[test]
    fn a_root_site_s_snapshots_are_at_the_domain_root() {
        assert_eq!(Prefix::default().snapshot_url(&Snapshot::Dev), "/dev/");
    }

    #[test]
    fn a_manifest_lists_releases_newest_first_once_each() {
        let m = Manifest::new([v("0.9.0"), v("0.10.0"), v("0.9.0"), v("0.2.1")]);
        assert_eq!(m.versions(), [v("0.10.0"), v("0.9.0"), v("0.2.1")]);
    }

    #[test]
    fn the_latest_release_is_the_newest() {
        assert_eq!(
            Manifest::new([v("0.9.0"), v("0.10.0")]).latest(),
            Some(v("0.10.0"))
        );
    }

    #[test]
    fn a_manifest_with_no_releases_has_no_latest() {
        assert_eq!(Manifest::default().latest(), None);
    }

    #[test]
    fn the_root_sends_visitors_to_the_newest_release() {
        let manifest = Manifest::new([v("0.9.0"), v("0.10.0")]);
        assert_eq!(manifest.root_snapshot(), Snapshot::Release(v("0.10.0")));
    }

    #[test]
    fn before_the_first_release_the_root_sends_visitors_to_dev() {
        assert_eq!(Manifest::default().root_snapshot(), Snapshot::Dev);
    }

    // The shape every snapshot deployed before pyle reads, through a struct
    // of two strings: renaming a field or writing `null` for "no release"
    // would break the switcher on every old snapshot at once. Exact
    // equality on purpose: a new field is a change to that shape, and
    // should be decided, not slipped in.
    #[test]
    fn a_manifest_is_written_in_the_shape_deployed_snapshots_read() {
        let m = Manifest::new([v("0.13.0"), v("0.14.0")]);
        assert_eq!(
            serde_json::to_value(&m).unwrap(),
            json!({"latest": "0.14.0", "versions": ["0.14.0", "0.13.0"]})
        );
    }

    #[test]
    fn a_manifest_with_no_releases_is_written_with_an_empty_latest() {
        assert_eq!(
            serde_json::to_value(Manifest::default()).unwrap(),
            json!({"latest": "", "versions": []})
        );
    }

    #[test]
    fn a_written_manifest_reads_back_the_same() {
        for m in [
            Manifest::new([v("0.13.0"), v("0.14.0")]),
            Manifest::default(),
        ] {
            let json = serde_json::to_string(&m).unwrap();
            assert_eq!(
                serde_json::from_str::<Manifest>(&json).unwrap(),
                m,
                "{json}"
            );
        }
    }

    // Bad entries sit before good ones, so a read that stopped at the first
    // bad entry would lose the rest of the list.
    #[test]
    fn reading_drops_a_string_that_is_not_a_version() {
        let m: Manifest =
            serde_json::from_value(json!({"versions": ["v0.13.0", "0.14.0", "0.12", "0.11.0"]}))
                .unwrap();
        assert_eq!(m.versions(), [v("0.14.0"), v("0.11.0")]);
    }

    #[test]
    fn reading_drops_an_entry_that_is_not_a_string() {
        let m: Manifest =
            serde_json::from_value(json!({"versions": [0.13, null, {"v": 1}, "0.14.0"]})).unwrap();
        assert_eq!(m.versions(), [v("0.14.0")]);
    }

    #[test]
    fn reading_ignores_a_latest_of_any_type() {
        for latest in [json!(null), json!(5), json!({}), json!([])] {
            let m: Manifest =
                serde_json::from_value(json!({"latest": latest, "versions": ["0.14.0"]}))
                    .unwrap_or_else(|e| panic!("latest {latest}: {e}"));
            assert_eq!(m.versions(), [v("0.14.0")], "latest {latest}");
        }
    }

    #[test]
    fn reading_a_versions_that_is_not_a_list_gives_no_releases() {
        for versions in [json!(null), json!("0.14.0"), json!({"0.14.0": true})] {
            let m: Manifest = serde_json::from_value(json!({"versions": versions}))
                .unwrap_or_else(|e| panic!("versions {versions}: {e}"));
            assert_eq!(m, Manifest::default(), "versions {versions}");
        }
    }

    // A hand-edited list can be in any order; reading must sort it, or
    // `latest` would be whatever came first.
    #[test]
    fn reading_takes_latest_from_the_versions_not_the_field() {
        let m: Manifest = serde_json::from_value(
            json!({"latest": "9.9.9", "versions": ["0.13.0", "0.14.0", "0.13.0"]}),
        )
        .unwrap();
        assert_eq!(m.versions(), [v("0.14.0"), v("0.13.0")]);
    }

    #[test]
    fn reading_a_manifest_with_missing_fields_gives_no_releases() {
        let m: Manifest = serde_json::from_value(json!({})).unwrap();
        assert_eq!(m, Manifest::default());
    }
}
