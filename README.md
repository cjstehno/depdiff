# Dependency Diff Tool

Used to compare the contents of a local maven repository with a remote repository, noting the jar and pom files missing
from the remote repository.

The use case for this tool is one where developers build against a controlled internal shared maven repository server. 
When new artifacts are brought in, they must be vetted and approved in order to be promoted into the shared repository.

The general workflow with this tool is to remove your local maven `repository` directory, build your project using external
open maven repositories and then run this tool against your internal shared repository to determine the differences - I don't
condone this behavior, but I do _have to_ work with it.

It reports on `.jar` and `.pom` files and ignores any `SNAPSHOT` artifact versions.

## Building

The tool is written in [Rust](https://www.rust-lang.org/). To build the project locally, clone or download the source and run:

    cargo build --release

in the project directory. The tool binary will be in the `target/release` directory. No installation is required.

## Usage

Running `depdiff -h` will generate the command help, which will be similar to the following:

```
Repository Diff Tool 0.1.0
Christopher J. Stehno <chris@stehno.com>
Determines the local dependencies missing from a remote repository.

USAGE:
    depdiff.exe [FLAGS] [OPTIONS] --local <LOCAL-PATH> --remote <REPO-URL>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v               Sets the level of verbosity

OPTIONS:
    -a, --archive <TAR_FILE>          The missing repo files will be archived in the specified tar file.
    -d, --display <DISPLAY-FORMAT>    Specifies the result display format to be used (SHORT, PATH, and LONG; defaults to
                                      LONG)
    -i, --ignore <GROUP>...           Ignores the artifacts under the specified group (in dot-form).
    -l, --local <LOCAL-PATH>          Path to local repository.
    -r, --remote <REPO-URL>           Remote repository URL
```