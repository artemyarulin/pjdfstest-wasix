use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use regex::Regex;
use serde::Serialize;

use crate::{RunError, fs};

static DESC_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?m)^desc=(?:"([^"]*)"|'([^']*)'|([^\r\n#]+))"#).unwrap());

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TestCase {
    pub group: String,
    pub path: PathBuf,
    pub name: String,
    pub desc: String,
}

pub fn discover(test_dir: &Path) -> Result<Vec<TestCase>, RunError> {
    let mut groups = fs::read_dir_sorted(test_dir)?;
    groups.retain(|entry| entry.path().is_dir());

    let mut tests = Vec::new();
    for group_entry in groups {
        let group_path = group_entry.path();
        let group = group_entry.file_name().to_string_lossy().into_owned();

        let mut files = fs::read_dir_sorted(&group_path)?;
        files.retain(|entry| entry.path().extension().is_some_and(|ext| ext == "t"));

        for file_entry in files {
            let path = file_entry.path();
            let name = path
                .file_stem()
                .map(|stem| stem.to_string_lossy().into_owned())
                .unwrap_or_default();
            let contents = fs::read_to_string(&path)?;
            let desc = extract_desc(&path, &contents)?;

            tests.push(TestCase {
                group: group.clone(),
                path,
                name,
                desc,
            });
        }
    }

    Ok(tests)
}

fn extract_desc(path: &Path, contents: &str) -> Result<String, RunError> {
    let captures = DESC_RE
        .captures(contents)
        .ok_or_else(|| RunError::MissingDesc {
            path: path.to_path_buf(),
        })?;
    let desc = captures
        .get(1)
        .or_else(|| captures.get(2))
        .or_else(|| captures.get(3))
        .map(|matched| matched.as_str().trim().to_string())
        .unwrap_or_default();
    Ok(desc)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::discover;

    #[test]
    fn discovers_test_cases() {
        let root = workspace_root().join("test_scenarious");
        let cases = discover(&root).expect("discover test cases");
        let snapshot = cases
            .iter()
            .map(|case| {
                format!(
                    "{}\t{}\t{}\t{}",
                    case.group,
                    relative_path(&root, &case.path).display(),
                    case.name,
                    case.desc
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        insta::assert_snapshot!(snapshot);
    }

    fn workspace_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .expect("workspace root")
            .to_path_buf()
    }

    fn relative_path<'a>(root: &Path, path: &'a Path) -> &'a Path {
        path.strip_prefix(root).unwrap_or(path)
    }
}
