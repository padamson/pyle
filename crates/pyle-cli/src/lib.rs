//! Deploys a static site one folder per version. [`add`] copies a build into
//! the site as `dev/` or `v<version>/`, and [`update`] writes the
//! `versions.json` that lists the releases and an `index.html` that
//! redirects to the newest.
//!
//! It works on a directory. Checking out the branch that serves the site
//! (`gh-pages`) and pushing it stay with the workflow.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use pyle_core::{MANIFEST_FILE, Manifest, Prefix, Snapshot, Version};

/// The file name of the root redirect, and of the page a build must have
/// for the redirect to land on.
pub const INDEX_FILE: &str = "index.html";

/// Why a deploy step failed, naming the path or input at fault.
#[derive(Debug)]
pub enum Error {
    /// A file operation on `path` failed.
    Io { path: PathBuf, source: io::Error },
    /// Copying the file `from` to `to` failed. Either end may be at fault:
    /// the operating system does not say which.
    Copy {
        from: PathBuf,
        to: PathBuf,
        source: io::Error,
    },
    /// The build, the site, or what sits where the snapshot goes is not a
    /// directory.
    NotADirectory(PathBuf),
    /// One of the build and the site holds the other, so deploying would
    /// copy the build into itself or replace part of it.
    Overlap { build: PathBuf, site: PathBuf },
    /// The build holds something other than a file or a folder, such as a
    /// symlink or a named pipe, which `add` does not copy.
    NotAFileOrFolder(PathBuf),
    /// The build has no `index.html` at its top, so the snapshot's URL
    /// would serve nothing.
    NoIndex(PathBuf),
    /// The site has no release folder and no `dev/`, so the root has
    /// nowhere to redirect.
    NothingDeployed(PathBuf),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
            Self::Copy { from, to, source } => write!(
                f,
                "copying {} to {}: {source}",
                from.display(),
                to.display()
            ),
            Self::NotADirectory(path) => write!(f, "{} is not a directory", path.display()),
            Self::Overlap { build, site } => write!(
                f,
                "the build {} and the site {} overlap; keep the build outside the site",
                build.display(),
                site.display()
            ),
            Self::NotAFileOrFolder(path) => write!(
                f,
                "{} is not a file or a folder; only files and folders are copied",
                path.display()
            ),
            Self::NoIndex(build) => write!(
                f,
                "the build {} has no {INDEX_FILE}; is it the built site?",
                build.display()
            ),
            Self::NothingDeployed(site) => write!(
                f,
                "the site {} has no v<version>/ or dev/ folder for the root to redirect to",
                site.display()
            ),
        }
    }
}

// No `source`: `Display` already prints the cause of an `Io` error, and a
// reporter that walks the chain would print it twice.
impl std::error::Error for Error {}

/// What [`add`] did: the snapshot's folder, and whether one was there
/// before.
#[derive(Debug, PartialEq, Eq)]
pub struct Added {
    pub dest: PathBuf,
    pub replaced: bool,
}

/// Replaces the `snapshot`'s folder under `site` with a copy of `build`.
///
/// The copy goes to a hidden folder beside the snapshot first. Once it is
/// whole, the old snapshot is moved aside, the copy is moved into place, and
/// only then is the old one deleted, so the site holds a whole snapshot
/// whichever step fails. An `add` that was stopped between the two moves
/// has its old snapshot put back by the next one.
pub fn add(build: &Path, site: &Path, snapshot: &Snapshot) -> Result<Added, Error> {
    let build = existing_dir(build)?;
    let site = existing_dir(site)?;
    if site.starts_with(&build) || build.starts_with(&site) {
        return Err(Error::Overlap { build, site });
    }
    if !build.join(INDEX_FILE).is_file() {
        return Err(Error::NoIndex(build));
    }
    let name = snapshot.dir_name();
    let dest = site.join(&name);
    let staging = site.join(format!(".{name}.pyle-new"));
    let aside = site.join(format!(".{name}.pyle-old"));

    if !present(&dest) && present(&aside) {
        fs::rename(&aside, &dest).map_err(io_at(&aside))?;
    }
    let replaced = match fs::symlink_metadata(&dest) {
        Ok(meta) if meta.is_dir() => true,
        Ok(_) => return Err(Error::NotADirectory(dest)),
        Err(_) => false,
    };
    remove_if_present(&staging)?;
    remove_if_present(&aside)?;

    let swapped = copy_dir(&build, &staging)
        .and_then(|()| swap(&staging, &dest, replaced.then_some(aside.as_path())));
    if let Err(err) = swapped {
        // Best effort: the error that stopped the swap is the one to report.
        let _ = fs::remove_dir_all(&staging);
        return Err(err);
    }
    remove_if_present(&aside)?;
    Ok(Added { dest, replaced })
}

/// Moves `staging` to `dest`. A folder already at `dest` goes to `aside`
/// first, and comes back if the move into place fails.
fn swap(staging: &Path, dest: &Path, aside: Option<&Path>) -> Result<(), Error> {
    if let Some(aside) = aside {
        fs::rename(dest, aside).map_err(io_at(dest))?;
    }
    fs::rename(staging, dest).map_err(|source| {
        if let Some(aside) = aside {
            // Best effort, as above.
            let _ = fs::rename(aside, dest);
        }
        Error::Io {
            path: dest.to_owned(),
            source,
        }
    })
}

/// What [`update`] wrote: the manifest, and the URL the root redirects to.
#[derive(Debug, PartialEq, Eq)]
pub struct Written {
    pub manifest: Manifest,
    pub redirect: String,
}

/// Writes the site's manifest and root redirect from the release folders
/// under it. A site with no release and no `dev/` is an error, and nothing
/// is written, since the redirect would lead nowhere.
pub fn update(site: &Path, prefix: &Prefix, title: &str) -> Result<Written, Error> {
    let manifest = scan(site)?;
    let root = manifest.root_snapshot();
    if !fs::symlink_metadata(site.join(root.dir_name())).is_ok_and(|meta| meta.is_dir()) {
        return Err(Error::NothingDeployed(site.to_owned()));
    }
    let json = serde_json::to_string_pretty(&manifest).expect("a manifest is strings only");
    write(&site.join(MANIFEST_FILE), json + "\n")?;
    let redirect = prefix.snapshot_url(&root);
    write(&site.join(INDEX_FILE), redirect_page(&redirect, title))?;
    Ok(Written { manifest, redirect })
}

/// The releases deployed under `site`: each folder named `v` and a version.
/// A symlink is not a folder here, as it is not one to [`add`].
fn scan(site: &Path) -> Result<Manifest, Error> {
    let mut versions = Vec::new();
    for entry in fs::read_dir(site).map_err(io_at(site))? {
        let entry = entry.map_err(io_at(site))?;
        let Some(version) = entry.file_name().to_str().and_then(Version::from_dir_name) else {
            continue;
        };
        if entry.file_type().map_err(io_at(&entry.path()))?.is_dir() {
            versions.push(version);
        }
    }
    Ok(Manifest::new(versions))
}

/// A page that sends the browser to `url` at once.
fn redirect_page(url: &str, title: &str) -> String {
    let title = html_escape::encode_text(title);
    let attr = html_escape::encode_double_quoted_attribute(url);
    let text = html_escape::encode_text(url);
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>{title}</title>
<meta http-equiv="refresh" content="0; url={attr}">
<link rel="canonical" href="{attr}">
</head>
<body>Redirecting to <a href="{attr}">{text}</a>…</body>
</html>
"#
    )
}

fn existing_dir(path: &Path) -> Result<PathBuf, Error> {
    let path = fs::canonicalize(path).map_err(io_at(path))?;
    if path.is_dir() {
        Ok(path)
    } else {
        Err(Error::NotADirectory(path))
    }
}

/// Whether anything is at `path`. `symlink_metadata`, so a dangling symlink
/// counts.
fn present(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok()
}

/// Removes the folder at `path`, if anything is there; a file there is an
/// error rather than removed.
fn remove_if_present(path: &Path) -> Result<(), Error> {
    if present(path) {
        fs::remove_dir_all(path).map_err(io_at(path))?;
    }
    Ok(())
}

fn copy_dir(from: &Path, to: &Path) -> Result<(), Error> {
    fs::create_dir(to).map_err(io_at(to))?;
    for entry in fs::read_dir(from).map_err(io_at(from))? {
        let entry = entry.map_err(io_at(from))?;
        let src = entry.path();
        let kind = entry.file_type().map_err(io_at(&src))?;
        let dst = to.join(entry.file_name());
        if kind.is_dir() {
            copy_dir(&src, &dst)?;
        } else if kind.is_file() {
            if let Err(source) = fs::copy(&src, &dst) {
                return Err(Error::Copy {
                    from: src,
                    to: dst,
                    source,
                });
            }
        } else {
            return Err(Error::NotAFileOrFolder(src));
        }
    }
    Ok(())
}

fn write(path: &Path, contents: String) -> Result<(), Error> {
    fs::write(path, contents).map_err(io_at(path))
}

fn io_at(path: &Path) -> impl FnOnce(io::Error) -> Error + '_ {
    move |source| Error::Io {
        path: path.to_owned(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn touch(path: &Path, contents: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    fn dir_with(dirs: &[&str]) -> TempDir {
        let dir = TempDir::new().unwrap();
        for name in dirs {
            fs::create_dir(dir.path().join(name)).unwrap();
        }
        dir
    }

    fn read(path: &Path) -> String {
        fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }

    /// The names in `dir`, hidden ones included, sorted.
    fn names(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    /// Asserts that `needle` appears in `page` exactly once.
    #[track_caller]
    fn once(page: &str, needle: &str) {
        assert_eq!(page.matches(needle).count(), 1, "{needle} in:\n{page}");
    }

    #[test]
    fn the_release_folders_list_newest_first() {
        let site = dir_with(&["v0.9.0", "v0.10.0", "v0.2.1"]);
        assert_eq!(
            scan(site.path()).unwrap().versions(),
            [
                Version::new(0, 10, 0),
                Version::new(0, 9, 0),
                Version::new(0, 2, 1)
            ]
        );
    }

    #[test]
    fn folders_that_are_not_releases_are_left_out() {
        let site = dir_with(&["dev", "vendor", "v0.1", "assets", "v1.0.0"]);
        assert_eq!(
            scan(site.path()).unwrap().versions(),
            [Version::new(1, 0, 0)]
        );
    }

    #[test]
    fn a_file_named_like_a_release_is_not_one() {
        let site = dir_with(&[]);
        touch(&site.path().join("v1.0.0"), "");
        assert_eq!(scan(site.path()).unwrap(), Manifest::default());
    }

    #[cfg(unix)]
    #[test]
    fn a_symlink_named_like_a_release_is_not_one() {
        let site = dir_with(&["v1.0.0"]);
        std::os::unix::fs::symlink("v1.0.0", site.path().join("v2.0.0")).unwrap();
        assert_eq!(
            scan(site.path()).unwrap().versions(),
            [Version::new(1, 0, 0)]
        );
    }

    #[test]
    fn scanning_a_missing_site_names_it() {
        let site = dir_with(&[]);
        let missing = site.path().join("gh");
        let scanned = scan(&missing);
        assert!(
            matches!(&scanned, Err(Error::Io { path, .. }) if *path == missing),
            "{scanned:?}"
        );
    }

    #[test]
    fn the_redirect_refreshes_to_the_url() {
        let page = redirect_page("/cqa-grader/v0.10.0/", "cqa-grader");
        once(&page, r#"content="0; url=/cqa-grader/v0.10.0/""#);
    }

    #[test]
    fn the_redirect_names_the_url_as_canonical() {
        let page = redirect_page("/cqa-grader/v0.10.0/", "cqa-grader");
        once(
            &page,
            r#"<link rel="canonical" href="/cqa-grader/v0.10.0/">"#,
        );
    }

    #[test]
    fn the_redirect_escapes_its_title() {
        let page = redirect_page("/", "a<b & c");
        once(&page, "<title>a&lt;b &amp; c</title>");
    }

    #[test]
    fn the_redirect_escapes_its_url() {
        let page = redirect_page(r#"/a"><script>/"#, "t");
        once(&page, r#"url=/a&quot;&gt;&lt;script&gt;/""#);
    }

    #[test]
    fn update_writes_the_manifest_of_the_release_folders() {
        let site = dir_with(&["dev", "v0.9.0", "v0.10.0"]);
        update(site.path(), &Prefix::new("/x"), "x").unwrap();
        assert_eq!(
            read(&site.path().join(MANIFEST_FILE)),
            "{\n  \"latest\": \"0.10.0\",\n  \"versions\": [\n    \"0.10.0\",\n    \"0.9.0\"\n  ]\n}\n"
        );
    }

    #[test]
    fn update_writes_a_root_that_redirects_under_the_prefix() {
        let site = dir_with(&["v0.10.0"]);
        update(site.path(), &Prefix::new("/cqa-grader"), "cqa-grader").unwrap();
        assert_eq!(
            read(&site.path().join(INDEX_FILE)),
            redirect_page("/cqa-grader/v0.10.0/", "cqa-grader")
        );
    }

    #[test]
    fn update_returns_what_it_wrote() {
        let site = dir_with(&["v1.0.0"]);
        assert_eq!(
            update(site.path(), &Prefix::new("/x"), "t").unwrap(),
            Written {
                manifest: Manifest::new([Version::new(1, 0, 0)]),
                redirect: "/x/v1.0.0/".into(),
            }
        );
    }

    #[test]
    fn before_the_first_release_update_redirects_to_dev() {
        let site = dir_with(&["dev"]);
        assert_eq!(
            update(site.path(), &Prefix::new("/x"), "t").unwrap(),
            Written {
                manifest: Manifest::default(),
                redirect: "/x/dev/".into(),
            }
        );
    }

    #[test]
    fn update_refuses_a_site_with_nothing_deployed() {
        let site = dir_with(&["assets"]);
        let updated = update(site.path(), &Prefix::new("/x"), "t");
        assert!(
            matches!(&updated, Err(Error::NothingDeployed(path)) if path == site.path()),
            "{updated:?}"
        );
    }

    #[test]
    fn a_file_named_dev_is_nothing_deployed() {
        let site = dir_with(&[]);
        touch(&site.path().join("dev"), "a file");
        let updated = update(site.path(), &Prefix::new("/x"), "t");
        assert!(
            matches!(updated, Err(Error::NothingDeployed(_))),
            "{updated:?}"
        );
    }

    #[test]
    fn a_refused_update_writes_nothing() {
        let site = dir_with(&["assets"]);
        update(site.path(), &Prefix::new("/x"), "t").unwrap_err();
        assert_eq!(names(site.path()), ["assets"]);
    }

    /// A build holding only its `index.html`.
    fn built() -> TempDir {
        let build = dir_with(&[]);
        touch(&build.path().join(INDEX_FILE), "new");
        build
    }

    fn canonical(path: &Path) -> PathBuf {
        fs::canonicalize(path).unwrap()
    }

    #[test]
    fn add_copies_the_build_into_the_snapshot_folder() {
        let build = built();
        touch(&build.path().join("pkg/site.wasm"), "wasm");
        let site = dir_with(&[]);
        add(build.path(), site.path(), &Snapshot::Dev).unwrap();
        assert_eq!(
            [
                read(&site.path().join("dev/index.html")),
                read(&site.path().join("dev/pkg/site.wasm"))
            ],
            ["new", "wasm"]
        );
    }

    #[test]
    fn add_returns_the_new_snapshot_folder() {
        let build = built();
        let site = dir_with(&[]);
        assert_eq!(
            add(build.path(), site.path(), &"0.15.0".parse().unwrap()).unwrap(),
            Added {
                dest: canonical(site.path()).join("v0.15.0"),
                replaced: false,
            }
        );
    }

    #[test]
    fn add_says_when_a_snapshot_was_there_before() {
        let build = built();
        let site = dir_with(&["dev"]);
        assert_eq!(
            add(build.path(), site.path(), &Snapshot::Dev).unwrap(),
            Added {
                dest: canonical(site.path()).join("dev"),
                replaced: true,
            }
        );
    }

    #[test]
    fn add_drops_what_the_old_snapshot_had() {
        let build = built();
        let site = dir_with(&[]);
        touch(&site.path().join("dev/stale.js"), "old");
        add(build.path(), site.path(), &Snapshot::Dev).unwrap();
        assert_eq!(names(&site.path().join("dev")), ["index.html"]);
    }

    #[test]
    fn add_leaves_the_other_snapshots_alone() {
        let build = built();
        let site = dir_with(&[]);
        touch(&site.path().join("v0.9.0/index.html"), "old release");
        add(build.path(), site.path(), &Snapshot::Dev).unwrap();
        assert_eq!(read(&site.path().join("v0.9.0/index.html")), "old release");
    }

    #[test]
    fn add_leaves_nothing_in_the_site_but_the_snapshot() {
        let build = built();
        let site = dir_with(&[]);
        add(build.path(), site.path(), &Snapshot::Dev).unwrap();
        assert_eq!(names(site.path()), ["dev"]);
    }

    #[test]
    fn replacing_a_snapshot_leaves_the_old_one_nowhere_in_the_site() {
        let build = built();
        let site = dir_with(&[]);
        touch(&site.path().join("dev/stale.js"), "old");
        add(build.path(), site.path(), &Snapshot::Dev).unwrap();
        assert_eq!(names(site.path()), ["dev"]);
    }

    #[test]
    fn add_clears_a_copy_an_earlier_add_left_behind() {
        let build = built();
        let site = dir_with(&[]);
        touch(&site.path().join(".dev.pyle-new/junk"), "");
        add(build.path(), site.path(), &Snapshot::Dev).unwrap();
        assert_eq!(names(site.path()), ["dev"]);
    }

    #[test]
    fn add_clears_an_old_snapshot_an_earlier_add_left_aside() {
        let build = built();
        let site = dir_with(&["dev"]);
        touch(&site.path().join(".dev.pyle-old/junk"), "");
        add(build.path(), site.path(), &Snapshot::Dev).unwrap();
        assert_eq!(names(site.path()), ["dev"]);
    }

    #[test]
    fn a_snapshot_an_interrupted_add_moved_aside_is_the_one_replaced() {
        let build = built();
        let site = dir_with(&[]);
        touch(&site.path().join(".dev.pyle-old/index.html"), "old");
        let added = add(build.path(), site.path(), &Snapshot::Dev).unwrap();
        assert!(added.replaced, "{added:?}");
    }

    #[track_caller]
    fn assert_overlap(added: &Result<Added, Error>, build: &Path, site: &Path) {
        let (b, s) = (canonical(build), canonical(site));
        assert!(
            matches!(added, Err(Error::Overlap { build, site }) if *build == b && *site == s),
            "{added:?}, expected the build {} and the site {}",
            b.display(),
            s.display()
        );
    }

    #[test]
    fn add_refuses_a_build_inside_the_site() {
        let site = dir_with(&[]);
        let build = site.path().join("dev/dist");
        touch(&build.join("index.html"), "page");
        let added = add(&build, site.path(), &Snapshot::Dev);
        assert_overlap(&added, &build, site.path());
    }

    #[test]
    fn a_refused_add_deletes_nothing() {
        let site = dir_with(&[]);
        let build = site.path().join("dev/dist");
        touch(&build.join("index.html"), "page");
        add(&build, site.path(), &Snapshot::Dev).unwrap_err();
        assert_eq!(read(&build.join("index.html")), "page");
    }

    #[test]
    fn a_refused_add_adds_nothing() {
        let site = dir_with(&[]);
        let build = site.path().join("dist");
        touch(&build.join("index.html"), "page");
        add(&build, site.path(), &Snapshot::Dev).unwrap_err();
        assert_eq!(names(site.path()), ["dist"]);
    }

    #[test]
    fn add_refuses_a_site_inside_the_build() {
        let build = dir_with(&["gh"]);
        let site = build.path().join("gh");
        let added = add(build.path(), &site, &Snapshot::Dev);
        assert_overlap(&added, build.path(), &site);
    }

    #[test]
    fn a_build_that_is_the_site_is_refused() {
        let site = dir_with(&[]);
        let added = add(site.path(), site.path(), &Snapshot::Dev);
        assert_overlap(&added, site.path(), site.path());
    }

    #[test]
    fn a_build_whose_name_extends_the_sites_is_beside_it() {
        let root = dir_with(&["gh", "gh-dist"]);
        touch(&root.path().join("gh-dist/index.html"), "page");
        let added = add(
            &root.path().join("gh-dist"),
            &root.path().join("gh"),
            &Snapshot::Dev,
        );
        assert!(added.is_ok(), "{added:?}");
    }

    #[test]
    fn add_refuses_a_build_that_is_a_file() {
        let site = dir_with(&[]);
        let build = site.path().join("dist.zip");
        touch(&build, "");
        let added = add(&build, site.path(), &Snapshot::Dev);
        assert!(
            matches!(&added, Err(Error::NotADirectory(path)) if *path == canonical(&build)),
            "{added:?}"
        );
    }

    #[test]
    fn add_names_a_missing_build() {
        let site = dir_with(&[]);
        let build = site.path().join("dist");
        let added = add(&build, site.path(), &Snapshot::Dev);
        assert!(
            matches!(&added, Err(Error::Io { path, .. }) if *path == build),
            "{added:?}"
        );
    }

    #[test]
    fn add_refuses_a_build_with_no_index_page() {
        let build = dir_with(&["pkg"]);
        let site = dir_with(&[]);
        let added = add(build.path(), site.path(), &Snapshot::Dev);
        assert!(
            matches!(&added, Err(Error::NoIndex(path)) if *path == canonical(build.path())),
            "{added:?}"
        );
    }

    #[test]
    fn a_folder_named_like_the_index_page_is_not_one() {
        let build = dir_with(&["index.html"]);
        let site = dir_with(&[]);
        let added = add(build.path(), site.path(), &Snapshot::Dev);
        assert!(matches!(added, Err(Error::NoIndex(_))), "{added:?}");
    }

    #[test]
    fn a_build_with_no_index_page_replaces_nothing() {
        let build = dir_with(&[]);
        let site = dir_with(&[]);
        touch(&site.path().join("dev/index.html"), "old");
        add(build.path(), site.path(), &Snapshot::Dev).unwrap_err();
        assert_eq!(read(&site.path().join("dev/index.html")), "old");
    }

    #[test]
    fn a_file_where_the_snapshot_goes_is_refused_by_path() {
        let build = built();
        let site = dir_with(&[]);
        touch(&site.path().join("dev"), "a file");
        let dest = canonical(site.path()).join("dev");
        let added = add(build.path(), site.path(), &Snapshot::Dev);
        assert!(
            matches!(&added, Err(Error::NotADirectory(path)) if *path == dest),
            "{added:?}"
        );
    }

    #[test]
    fn a_file_where_the_snapshot_goes_is_left_as_it_was() {
        let build = built();
        let site = dir_with(&[]);
        touch(&site.path().join("dev"), "a file");
        add(build.path(), site.path(), &Snapshot::Dev).unwrap_err();
        assert_eq!(
            (names(site.path()), read(&site.path().join("dev"))),
            (vec!["dev".to_string()], "a file".to_string())
        );
    }

    #[test]
    fn a_failed_move_into_place_puts_the_old_snapshot_back() {
        let site = dir_with(&[]);
        let dest = site.path().join("dev");
        touch(&dest.join("index.html"), "old");
        let aside = site.path().join(".dev.pyle-old");
        swap(&site.path().join("no-copy"), &dest, Some(&aside)).unwrap_err();
        assert_eq!(
            (names(site.path()), read(&dest.join("index.html"))),
            (vec!["dev".to_string()], "old".to_string())
        );
    }

    #[test]
    fn a_failed_move_into_place_names_the_snapshot_folder() {
        let site = dir_with(&[]);
        let dest = site.path().join("dev");
        let swapped = swap(&site.path().join("no-copy"), &dest, None);
        assert!(
            matches!(&swapped, Err(Error::Io { path, .. }) if *path == dest),
            "{swapped:?}"
        );
    }

    /// Adds a build as `dev` over a snapshot holding a folder that cannot
    /// be emptied, so the old snapshot cannot be deleted. Returns the site
    /// and what `add` returned.
    #[cfg(unix)]
    fn add_over_a_snapshot_that_cannot_be_deleted() -> (TempDir, Result<Added, Error>) {
        use std::os::unix::fs::PermissionsExt;
        let build = built();
        let site = dir_with(&[]);
        touch(&site.path().join("dev/locked/old.js"), "old");
        let set_mode = |dir: &str, mode| {
            fs::set_permissions(site.path().join(dir), fs::Permissions::from_mode(mode))
        };
        set_mode("dev/locked", 0o555).unwrap();
        let added = add(build.path(), site.path(), &Snapshot::Dev);
        // Unlocked wherever it ended up, so the temporary directory can go.
        for dir in ["dev/locked", ".dev.pyle-old/locked"] {
            let _ = set_mode(dir, 0o755);
        }
        (site, added)
    }

    #[cfg(unix)]
    #[test]
    fn an_old_snapshot_that_cannot_be_deleted_is_still_replaced_whole() {
        let (site, added) = add_over_a_snapshot_that_cannot_be_deleted();
        assert_eq!(names(&site.path().join("dev")), ["index.html"], "{added:?}");
    }

    #[cfg(unix)]
    #[test]
    fn an_old_snapshot_that_cannot_be_deleted_is_reported_by_path() {
        let (site, added) = add_over_a_snapshot_that_cannot_be_deleted();
        let aside = canonical(site.path()).join(".dev.pyle-old");
        assert!(
            matches!(&added, Err(Error::Io { path, .. }) if path.starts_with(&aside)),
            "{added:?}"
        );
    }

    /// A build holding a symlink, which `add` refuses part way through the
    /// copy, after `index.html` may already be copied.
    fn build_with_a_symlink() -> TempDir {
        let build = built();
        let page = build.path().join("index.html");
        let link = build.path().join("link");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&page, link).unwrap();
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&page, link).unwrap();
        build
    }

    #[test]
    #[cfg_attr(
        windows,
        ignore = "needs the symlink privilege; run with cargo nextest run -p pyle-cli --run-ignored only"
    )]
    fn add_refuses_a_symlink_in_the_build() {
        let build = build_with_a_symlink();
        let site = dir_with(&[]);
        let added = add(build.path(), site.path(), &Snapshot::Dev);
        assert!(
            matches!(&added, Err(Error::NotAFileOrFolder(path)) if path.ends_with("link")),
            "{added:?}"
        );
    }

    #[test]
    #[cfg_attr(
        windows,
        ignore = "needs the symlink privilege; run with cargo nextest run -p pyle-cli --run-ignored only"
    )]
    fn a_failed_copy_leaves_the_old_snapshot_deployed() {
        let build = build_with_a_symlink();
        let site = dir_with(&[]);
        touch(&site.path().join("dev/index.html"), "old");
        add(build.path(), site.path(), &Snapshot::Dev).unwrap_err();
        assert_eq!(read(&site.path().join("dev/index.html")), "old");
    }

    #[test]
    #[cfg_attr(
        windows,
        ignore = "needs the symlink privilege; run with cargo nextest run -p pyle-cli --run-ignored only"
    )]
    fn a_failed_copy_leaves_no_partial_copy_behind() {
        let build = build_with_a_symlink();
        let site = dir_with(&[]);
        add(build.path(), site.path(), &Snapshot::Dev).unwrap_err();
        assert_eq!(names(site.path()), Vec::<String>::new());
    }

    #[test]
    #[cfg_attr(
        windows,
        ignore = "needs the symlink privilege; run with cargo nextest run -p pyle-cli --run-ignored only"
    )]
    fn a_failed_copy_puts_back_a_snapshot_an_interrupted_add_moved_aside() {
        let build = build_with_a_symlink();
        let site = dir_with(&[]);
        touch(&site.path().join(".dev.pyle-old/index.html"), "old");
        add(build.path(), site.path(), &Snapshot::Dev).unwrap_err();
        assert_eq!(
            (
                names(site.path()),
                read(&site.path().join("dev/index.html"))
            ),
            (vec!["dev".to_string()], "old".to_string())
        );
    }

    #[test]
    fn each_error_names_its_path() {
        let path = PathBuf::from("/site/dev");
        let errors = [
            Error::Io {
                path: path.clone(),
                source: io::Error::other("boom"),
            },
            Error::Copy {
                from: path.clone(),
                to: "/t".into(),
                source: io::Error::other("boom"),
            },
            Error::Copy {
                from: "/f".into(),
                to: path.clone(),
                source: io::Error::other("boom"),
            },
            Error::NotADirectory(path.clone()),
            Error::Overlap {
                build: "/b".into(),
                site: path.clone(),
            },
            Error::Overlap {
                build: path.clone(),
                site: "/s".into(),
            },
            Error::NotAFileOrFolder(path.clone()),
            Error::NoIndex(path.clone()),
            Error::NothingDeployed(path),
        ];
        for err in errors {
            let shown = err.to_string();
            assert!(shown.contains("/site/dev"), "{err:?} shows {shown:?}");
        }
    }
}
