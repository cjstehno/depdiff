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
                let dependency = parse_dependency(&entry);
                trace!("{:?} --> {:?}", entry, dependency);
                dependencies.push(dependency);
            }
        }
    }

    info!("Found {} dependencies.", dependencies.len());

    dependencies
}

fn parse_dependency(file: &DirEntry) -> Dependency {
    // DirEntry("c:/Users/stehnoc/.m2/repository\\ch\\qos\\logback\\logback-classic\\1.1.3\\logback-classic-1.1.3.jar")
    //           |-- remove --------------------|-- group ---------|-- artifact ----|- ver -|- artifact ---|- v -|- typ
    // <storage-dir-path>/<group-dirs>/<artifact-name>/<version>/<artifact>-<version>-<classifier>.<type>
    // <storage-dir-path>/<group-dirs>/<artifact-name>/<version>/<artifact>-<version>.<type>

    Dependency {
        group: String::from(""),
        artifact: String::from(""),
        version: String::from(""),
        classifier: String::from(""),
        dep_type: String::from(""),
    }
}

fn is_dependency_file(file: &DirEntry) -> bool {
    file.file_name().to_str().unwrap_or("").ends_with(".pom") || file.file_name().to_str().unwrap_or("").ends_with(".jar")
}

/*
    scan local repo (with ignored-groups)
    - find all file paths ending in .pom or .jar
    - collect Dependency objects
    (these are the dependencies in my local repo)

    verify agaisnt remote repo
    - each dependency make HEAD request to remote repo
    - collect missing dependencies

    generate report

    ===
[INFO] Dep: DirEntry("c:/Users/stehnoc/.m2/repository\\ch\\qos\\logback\\logback-classic\\1.1.3\\logback-classic-1.1.3.jar")
[INFO] Dep: DirEntry("c:/Users/stehnoc/.m2/repository\\ch\\qos\\logback\\logback-classic\\1.1.3\\logback-classic-1.1.3.pom")
[INFO] Dep: DirEntry("c:/Users/stehnoc/.m2/repository\\cglib\\cglib-nodep\\3.1\\cglib-nodep-3.1.jar")
[INFO] Dep: DirEntry("c:/Users/stehnoc/.m2/repository\\cglib\\cglib-nodep\\3.1\\cglib-nodep-3.1.pom")
[INFO] Dep: DirEntry("c:/Users/stehnoc/.m2/repository\\backport-util-concurrent\\backport-util-concurrent\\3.1\\backport-util-concurrent-3.1.jar")
[INFO] Dep: DirEntry("c:/Users/stehnoc/.m2/repository\\backport-util-concurrent\\backport-util-concurrent\\3.1\\backport-util-concurrent-3.1.pom")
[INFO] Dep: DirEntry("c:/Users/stehnoc/.m2/repository\\avalon-framework\\avalon-framework\\4.1.3\\avalon-framework-4.1.3.pom")
[INFO] Dep: DirEntry("c:/Users/stehnoc/.m2/repository\\asm\\asm-util\\3.2\\asm-util-3.2.jar")
[INFO] Dep: DirEntry("c:/Users/stehnoc/.m2/repository\\asm\\asm-util\\3.2\\asm-util-3.2.pom")
[INFO] Dep: DirEntry("c:/Users/stehnoc/.m2/repository\\asm\\asm-tree\\3.3.1\\asm-tree-3.3.1.jar")
[INFO] Dep: DirEntry("c:/Users/stehnoc/.m2/repository\\asm\\asm-tree\\3.3.1\\asm-tree-3.3.1.pom")
[INFO] Dep: DirEntry("c:/Users/stehnoc/.m2/repository\\asm\\asm-tree\\3.2\\asm-tree-3.2.pom")
[INFO] Dep: DirEntry("c:/Users/stehnoc/.m2/repository\\asm\\asm-parent\\3.3.1\\asm-parent-3.3.1.pom")
[INFO] Dep: DirEntry("c:/Users/stehnoc/.m2/repository\\asm\\asm-parent\\3.2\\asm-parent-3.2.pom")
*/