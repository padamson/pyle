use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};
use pyle_cli::{Error, INDEX_FILE};
use pyle_core::{MANIFEST_FILE, Prefix, Snapshot};

/// Deploy a static site one folder per version, into a directory such as
/// a gh-pages checkout.
#[derive(Parser)]
#[command(name = "pyle", version = env!("CRATE_VERSION_WITH_BUILD"))]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Copy a build into the site as `dev/` or `v<version>/`, then rewrite
    /// the manifest and the root redirect
    Add {
        /// The built site to copy, such as Trunk's `dist/`
        build: PathBuf,
        /// `dev`, or a release as MAJOR.MINOR.PATCH with no `v`
        #[arg(long = "as", value_name = "SNAPSHOT")]
        snapshot: Snapshot,
        #[command(flatten)]
        site: Site,
    },
    /// Rewrite versions.json and the root redirect from the site's
    /// `v<version>/` folders
    Manifest {
        #[command(flatten)]
        site: Site,
    },
}

#[derive(Args)]
struct Site {
    /// The deployed site, such as a gh-pages checkout
    #[arg(long = "site", value_name = "DIR")]
    dir: PathBuf,
    /// The path the site is served under: `/<repo>` for GitHub project
    /// Pages, `/` for a domain's root
    #[arg(long)]
    prefix: Prefix,
    /// The title of the root redirect page
    #[arg(long)]
    title: String,
}

fn main() -> ExitCode {
    match run(Cli::parse().command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("pyle: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run(command: Command) -> Result<(), Error> {
    let site = match command {
        Command::Add {
            build,
            snapshot,
            site,
        } => {
            let added = pyle_cli::add(&build, &site.dir, &snapshot)?;
            let (build, dest) = (build.display(), added.dest.display());
            if added.replaced {
                println!("replaced {dest} with {build}");
            } else {
                println!("copied {build} to {dest}");
            }
            site
        }
        Command::Manifest { site } => site,
    };
    let written = pyle_cli::update(&site.dir, &site.prefix, &site.title)?;
    let versions = written.manifest.versions();
    if versions.is_empty() {
        println!("{MANIFEST_FILE} lists no releases");
    } else {
        let list: Vec<String> = versions.iter().map(ToString::to_string).collect();
        println!("{MANIFEST_FILE} lists {}", list.join(", "));
    }
    println!("{INDEX_FILE} redirects to {}", written.redirect);
    Ok(())
}
