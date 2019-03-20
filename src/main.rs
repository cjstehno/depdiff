#[macro_use]
extern crate clap;
extern crate fern;
#[macro_use]
extern crate log;
extern crate rayon;

use std::fs::{DirEntry, File};
use std::path::Path;
use std::time::Instant;

use clap::{App, Values};
use rayon::prelude::*;
use reqwest::Client;
use tar::Builder;

use dependency::Dependency;

use crate::dependency::DisplayFormat;

mod dependency;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let local_path = matches.value_of("local").unwrap();
    let remote_url = matches.value_of("remote").unwrap();
    let display_fmt = DisplayFormat::from(match matches.value_of("display") {
        Some(d) => d,
        None => "LONG"
    });

    configure_logging(matches.occurrences_of("verbose"));

    let started = Instant::now();
    let local_dependencies = scan_local(Path::new(local_path), matches.values_of("ignore"));

    let http_client = Client::new();

    println!("Dependencies in local ({}) missing from remote ({})...", local_path, remote_url);

    let missing_deps = local_dependencies.par_iter().filter(|dep| !in_remote_repo(&http_client, remote_url, dep)).collect::<Vec<&Dependency>>();

    for dep in &missing_deps {
        println!("Missing: {}", dep.to_display(&display_fmt));
    }

    let runtime = started.elapsed().as_secs();
    println!("[Missing {} of {} dependencies ({}s)]", &missing_deps.len(), local_dependencies.len(), runtime);

    // FIXME: move to function
    match matches.value_of("archive") {
        Some(tar_path) => {
            println!("Archiving missing files to: {}", tar_path);

            let mut ordered_missing = missing_deps.iter().map(|dep| { dep.to_display(&DisplayFormat::Path) }).collect::<Vec<String>>();
            ordered_missing.sort();

            archive_missing(Path::new(local_path), Path::new(tar_path), &ordered_missing);
        }
        None => ()
    }
}

fn archive_missing(local_path: &Path, arc_path: &Path, file_paths: &Vec<String>) {
    info!("Writing archive file ({:?})...", arc_path);

    let arc_file = File::create(arc_path).expect("Unable to create archive!");
    let mut tar_builder = Builder::new(arc_file);

    for item in file_paths {
        let full_path = local_path.join(Path::new(&item));
        debug!("Archiving file: {:?}", &full_path);

        match File::open(&full_path) {
            Ok(mut f) => {
                tar_builder.append_file(&item, &mut f).unwrap();
            }
            Err(_e) => warn!("Unable to open file ({:?}) - omitted from archive.", &full_path)
        }
    }

    tar_builder.finish().expect("Problem finishing tar file.");
}

fn configure_logging(verbosity: u64) {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!("[{}] {}", record.level(), message))
        })
        .level(match verbosity {
            0 => log::LevelFilter::Warn,
            1 => log::LevelFilter::Info,
            2 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace
        })
        .chain(std::io::stdout())
        .apply()
        .unwrap();
}

fn in_remote_repo(client: &Client, remote_url: &str, dependency: &Dependency) -> bool {
    debug!("Checking remote repo ({}) for dependency ({:?})...", remote_url, dependency);

    let response = client.head(&format!("{}/{}", remote_url, dependency.to_url_path())).send().unwrap();
    response.status().is_success()
}

fn scan_local(local_path: &Path, ignored_groups: Option<Values>) -> Vec<Dependency> {
    info!("Scanning local-path ({})...", local_path.to_str().unwrap_or(""));

    let ignored = ignored_groups.unwrap_or(Values::default()).collect::<Vec<&str>>();
    info!("Ignoring artifacts in group ({:?})", &ignored);

    let mut dependencies = vec![];

    let mut directories = vec![local_path.to_path_buf()];
    while !directories.is_empty() {
        for dir_entry in directories.pop().unwrap().read_dir().unwrap() {
            let entry = dir_entry.unwrap();

            if entry.file_type().unwrap().is_dir() {
                directories.push(entry.path());
            } else if is_dependency_file(&entry) {
                match Dependency::parse(entry.path().to_str().unwrap_or(""), local_path.to_str().unwrap_or(""), &ignored) {
                    Some(dep) => dependencies.push(dep),
                    None => ()
                }
            }
        }
    }

    info!("Found {} dependencies in local repository.", dependencies.len());

    dependencies
}

fn is_dependency_file(file: &DirEntry) -> bool {
    file.file_name().to_str().unwrap_or("").ends_with(".pom") || file.file_name().to_str().unwrap_or("").ends_with(".jar")
}
