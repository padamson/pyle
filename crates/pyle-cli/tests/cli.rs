use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

/// A build and an empty site, each in its own temporary directory.
struct Deploy {
    build: TempDir,
    site: TempDir,
}

impl Deploy {
    fn new() -> Self {
        let build = TempDir::new().unwrap();
        fs::write(build.path().join("index.html"), "page").unwrap();
        Self {
            build,
            site: TempDir::new().unwrap(),
        }
    }

    fn add(&self, snapshot: &str, prefix: &str) -> Output {
        run(pyle()
            .arg("add")
            .arg(self.build.path())
            .args(["--as", snapshot])
            .args(site_args(self.site.path(), prefix)))
    }

    fn manifest(&self, site: &Path) -> Output {
        run(pyle().arg("manifest").args(site_args(site, "/")))
    }

    fn read(&self, file: &str) -> String {
        let path = self.site.path().join(file);
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }
}

fn pyle() -> Command {
    Command::new(env!("CARGO_BIN_EXE_pyle"))
}

fn site_args(site: &Path, prefix: &str) -> [std::ffi::OsString; 6] {
    [
        "--site".into(),
        site.into(),
        "--prefix".into(),
        prefix.into(),
        "--title".into(),
        "site".into(),
    ]
}

fn run(command: &mut Command) -> Output {
    command.output().unwrap()
}

fn report(out: &Output) -> String {
    format!(
        "status {}\nstdout:\n{}\nstderr:\n{}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// Asserts that the run succeeded, and returns it.
#[track_caller]
fn ok(out: Output) -> Output {
    assert!(out.status.success(), "{}", report(&out));
    out
}

fn is_empty(dir: &Path) -> bool {
    fs::read_dir(dir).unwrap().next().is_none()
}

#[test]
fn adding_a_release_deploys_its_build() {
    let deploy = Deploy::new();
    ok(deploy.add("0.15.0", "/cqa-grader"));
    assert_eq!(deploy.read("v0.15.0/index.html"), "page");
}

#[test]
fn adding_a_release_lists_it() {
    let deploy = Deploy::new();
    ok(deploy.add("0.15.0", "/cqa-grader"));
    assert_eq!(
        deploy.read("versions.json"),
        "{\n  \"latest\": \"0.15.0\",\n  \"versions\": [\n    \"0.15.0\"\n  ]\n}\n"
    );
}

#[test]
fn adding_a_release_points_the_root_at_it_under_the_prefix() {
    let deploy = Deploy::new();
    ok(deploy.add("0.15.0", "/cqa-grader"));
    let root = deploy.read("index.html");
    assert_eq!(
        root.matches("url=/cqa-grader/v0.15.0/\"").count(),
        1,
        "{root}"
    );
}

#[test]
fn adding_a_release_reports_where_the_root_now_redirects() {
    let deploy = Deploy::new();
    let out = ok(deploy.add("0.15.0", "/cqa-grader"));
    assert!(
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .any(|l| l == "index.html redirects to /cqa-grader/v0.15.0/"),
        "{}",
        report(&out)
    );
}

/// The first word of each line a run printed.
fn first_words(out: &Output) -> Vec<String> {
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| l.split(' ').next().map(str::to_string))
        .collect()
}

#[test]
fn adding_a_new_snapshot_says_it_was_copied() {
    let deploy = Deploy::new();
    let out = ok(deploy.add("0.15.0", "/"));
    assert_eq!(first_words(&out)[0], "copied", "{}", report(&out));
}

#[test]
fn adding_a_snapshot_again_says_it_was_replaced() {
    let deploy = Deploy::new();
    ok(deploy.add("0.15.0", "/"));
    let out = ok(deploy.add("0.15.0", "/"));
    assert_eq!(first_words(&out)[0], "replaced", "{}", report(&out));
}

#[test]
fn a_site_with_nothing_deployed_fails_and_writes_no_redirect() {
    let deploy = Deploy::new();
    let out = deploy.manifest(deploy.site.path());
    assert_eq!(out.status.code(), Some(1), "{}", report(&out));
    assert!(is_empty(deploy.site.path()), "{}", report(&out));
}

#[test]
fn adding_dev_before_any_release_points_the_root_at_dev() {
    let deploy = Deploy::new();
    ok(deploy.add("dev", "/"));
    let root = deploy.read("index.html");
    assert_eq!(root.matches("url=/dev/\"").count(), 1, "{root}");
}

#[test]
fn a_v_before_the_version_is_a_usage_error_that_touches_nothing() {
    let deploy = Deploy::new();
    let out = deploy.add("v0.15.0", "/");
    assert_eq!(out.status.code(), Some(2), "{}", report(&out));
    assert!(is_empty(deploy.site.path()), "{}", report(&out));
}

#[test]
fn a_prefix_that_is_a_url_is_a_usage_error_that_touches_nothing() {
    let deploy = Deploy::new();
    let out = deploy.add("dev", "https://padamson.github.io/cqa-grader");
    assert_eq!(out.status.code(), Some(2), "{}", report(&out));
    assert!(is_empty(deploy.site.path()), "{}", report(&out));
}

#[test]
fn a_missing_site_fails_and_names_it() {
    let deploy = Deploy::new();
    let missing = deploy.site.path().join("gh");
    let out = deploy.manifest(&missing);
    assert_eq!(out.status.code(), Some(1), "{}", report(&out));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains(&*missing.to_string_lossy()),
        "{}",
        report(&out)
    );
}

#[test]
fn manifest_lists_the_releases_already_in_the_site() {
    let deploy = Deploy::new();
    for dir in ["v0.9.0", "v0.10.0", "dev"] {
        fs::create_dir(deploy.site.path().join(dir)).unwrap();
    }
    ok(deploy.manifest(deploy.site.path()));
    assert_eq!(
        deploy.read("versions.json"),
        "{\n  \"latest\": \"0.10.0\",\n  \"versions\": [\n    \"0.10.0\",\n    \"0.9.0\"\n  ]\n}\n"
    );
}
