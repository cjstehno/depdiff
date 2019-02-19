#[derive(Debug, PartialEq)]
pub struct Dependency {
    pub group: String,
    pub artifact: String,
    pub version: String,
    pub classifier: Option<String>,
    pub dep_type: String,
}

impl Dependency {
    pub fn parse(path: &str, repo_root: &str) -> Dependency {
        // normalize the path string
        // FIXME: may need to make sure no leading slash
        let path = path.replace("\\", "/").replace(repo_root.replace("\\", "/").as_str(), "");
        let path = if path.starts_with("/") { &path[1..] } else { path.as_str() };

        // patterns:
        // <group-dirs>/<artifact-name>/<version>/<artifact>-<version>-<classifier>.<type>
        // <group-dirs>/<artifact-name>/<version>/<artifact>-<version>.<type>

        let parts = path.split("/").collect::<Vec<&str>>();

        let group = parts[..(parts.len() - 3)].join(".");
        let artifact = parts[parts.len() - 3];   // this is the artifact name
        let version = parts[parts.len() - 2];    // this is the version
        let file_part = parts[parts.len() - 1];  // this is the artifact-version-classifier.type part

        let classifier_type = file_part.replace(&format!("{}-{}", artifact, version), "");
        let (classifier, dep_type) = if classifier_type.starts_with("-") {
            // has classifier
            let c_and_t = classifier_type.split(".").collect::<Vec<&str>>();
            (c_and_t[0][1..].to_string(), c_and_t[1].to_string())
        } else {
            // no classifier
            ("".to_string(), classifier_type[1..].to_string())
        };

        let dependency = Dependency {
            group,
            artifact: artifact.to_string(),
            version: version.to_string(),
            classifier: if classifier != "" { Some(classifier) } else { None },
            dep_type,
        };

        trace!("{:?} --> {:?}", path, dependency);

        dependency
    }

    pub fn to_url_path(&self) -> String {
        let group_path = self.group.replace(".", "/");
        match &self.classifier {
            Some(c) => format!("{}/{}/{}/{}-{}-{}.{}", group_path, &self.artifact, &self.version, &self.artifact, &self.version, c, &self.dep_type),
            None => format!("{}/{}/{}/{}-{}.{}", group_path, &self.artifact, &self.version, &self.artifact, &self.version, &self.dep_type)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dependency::Dependency;

    #[test]
    fn to_path_without_classifier(){
        let dep = Dependency {
            group: String::from("org.something.else"),
            artifact: String::from("frog-pond"),
            version: String::from("1.3.4"),
            classifier: None,
            dep_type: String::from("jar"),
        };

        assert_eq!(dep.to_url_path(), "org/something/else/frog-pond/1.3.4/frog-pond-1.3.4.jar");
    }

    #[test]
    fn to_path_with_classifier(){
        let dep = Dependency {
            group: String::from("org.something.else"),
            artifact: String::from("frog-pond"),
            version: String::from("1.3.4"),
            classifier: Some(String::from("docs")),
            dep_type: String::from("jar"),
        };

        assert_eq!(dep.to_url_path(), "org/something/else/frog-pond/1.3.4/frog-pond-1.3.4-docs.jar");
    }

    #[test]
    fn parse_dep_with_classifier() {
        let dep = Dependency::parse("/foo/bar/org/something/else/frog-pond/1.3.4/frog-pond-1.3.4-sources.jar", "/foo/bar");
        assert_eq!(dep, Dependency {
            group: String::from("org.something.else"),
            artifact: String::from("frog-pond"),
            version: String::from("1.3.4"),
            classifier: Some(String::from("sources")),
            dep_type: String::from("jar"),
        });
    }

    #[test]
    fn parse_dep_without_classifier_no_leading_slash() {
        let dep = Dependency::parse("/foo/bar/com/fasterxml/classmate/1.3.4/classmate-1.3.4.jar", "/foo/bar");
        assert_eq!(dep, Dependency {
            group: String::from("com.fasterxml"),
            artifact: String::from("classmate"),
            version: String::from("1.3.4"),
            classifier: None,
            dep_type: String::from("jar"),
        });
    }

    #[test]
    fn parse_dep_without_classifier_with_leading_slash() {
        let dep = Dependency::parse("/foo/bar/com/fasterxml/classmate/1.3.4/classmate-1.3.4.jar", "/foo/bar/");
        assert_eq!(dep, Dependency {
            group: String::from("com.fasterxml"),
            artifact: String::from("classmate"),
            version: String::from("1.3.4"),
            classifier: None,
            dep_type: String::from("jar"),
        });
    }

    #[test]
    fn parse_dep_short_group() {
        let dep = Dependency::parse("/bing/bong/antlr/antlr/2.7.7/antlr-2.7.7.pom", "/bing/bong");
        assert_eq!(dep, Dependency {
            group: String::from("antlr"),
            artifact: String::from("antlr"),
            version: String::from("2.7.7"),
            classifier: None,
            dep_type: String::from("pom"),
        });
    }
}