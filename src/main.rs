extern crate clap;
extern crate fern;
#[macro_use]
extern crate log;

use std::fs::DirEntry;
use std::path::Path;

use clap::{App, Arg};

#[derive(Debug)]
struct Dependency {
    group: String,
    artifact: String,
    version: String,
    classifier: String,
    dep_type: String,
}

fn main() {
    let matches = App::new("Repository Checking Tool")
        .version("0.0.1")
        .author("Christopher J. Stehno <chris@stehno.com>")
        .about("Determines the local dependencies missing from a remote repository.")
        .arg(Arg::with_name("verbose").long("verbose").short("v").multiple(true).help("Turns on verbose operation logging information."))
        .arg(Arg::with_name("local").long("local").short("l").value_name("LOCAL-PATH").help("Path to local repository.").required(true).takes_value(true))
        .arg(Arg::with_name("remote").long("remote").short("r").value_name("REMOTE-URL").help("Remote repository URL.").required(true).takes_value(true))
        .get_matches();

    let local_path = matches.value_of("local").unwrap();
    let remote_url = matches.value_of("remote").unwrap();

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!("[{}] {}", record.level(), message))
        })
        .level(match matches.occurrences_of("verbose") {
            0 => log::LevelFilter::Info,
            1 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace
        })
        .chain(std::io::stdout())
        .apply()
        .unwrap();

    let local_dependencies = scan_local(Path::new(local_path));
}

fn scan_local(local_path: &Path) -> Vec<Dependency> {
    info!("Scanning local-path ({})...", local_path.to_str().unwrap_or(""));

    let mut dependencies = vec![];

    let mut directories = vec![local_path.to_path_buf()];
    while !directories.is_empty() {
        for dir_entry in directories.pop().unwrap().read_dir().unwrap() {
            let entry = dir_entry.unwrap();

            if entry.file_type().unwrap().is_dir() {
                directories.push(entry.path());
            } else if is_dependency_file(&entry) {
                dependencies.push(parse_dependency(entry.path().to_str().unwrap_or(""), local_path.to_str().unwrap_or("")));
            }
        }
    }

    info!("Found {} dependencies.", dependencies.len());

    dependencies
}

fn parse_dependency(path: &str, repo_root: &str) -> Dependency {
    // normalize the path string
    // FIXME: may need to make sure no leading slash
    let path = path.replace("\\", "/").replace(repo_root.replace("\\", "/").as_str(), "");

    // expected patterns
    // <group-dirs>/<artifact-name>/<version>/<artifact>-<version>-<classifier>.<type>
    // <group-dirs>/<artifact-name>/<version>/<artifact>-<version>.<type>

    let parts = path.split("/").collect::<Vec<&str>>();

    let group = parts[..(parts.len() - 4)].join(".");
    let artifact = parts[parts.len() - 3];   // this is the artifact name
    let version = parts[parts.len() - 2];    // this is the version
    let file_part = parts[parts.len() - 1];       // this is the artifact-version-classifier.type part

    // FIXME: probably make the classifier Optional
    // FIXME: move this parse function into Dependency impl

    let tempt = format!("{}-{}", artifact, version);
    let classifier_type = file_part.replace(&tempt, "");
    let (classifier, dep_type) = if classifier_type.starts_with("-") {
        let c_and_t = classifier_type.split(".").collect::<Vec<&str>>();
        (c_and_t[0][1..].to_string(), c_and_t[1].to_string())
    } else {
        ("".to_string(), classifier_type[1..].to_string())
    };

    let dependency = Dependency {
        group,
        artifact: artifact.to_string(),
        version: version.to_string(),
        classifier,
        dep_type,
    };

    trace!("{:?} --> {:?}", path, dependency);

    dependency
}

fn is_dependency_file(file: &DirEntry) -> bool {
    file.file_name().to_str().unwrap_or("").ends_with(".pom") || file.file_name().to_str().unwrap_or("").ends_with(".jar")
}

/* TODO: wip
    verify agaisnt remote repo
    - each dependency make HEAD request to remote repo
    - collect missing dependencies

    generate report
*/