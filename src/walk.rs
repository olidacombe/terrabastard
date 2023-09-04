use std::{
    collections::HashSet,
    iter::once,
    path::{Path, PathBuf},
};

use ignore::{DirEntry, WalkBuilder};
use indexmap::IndexMap;

use crate::terraform::{is_top_level, parse, strings};

pub fn is_file(e: &DirEntry) -> bool {
    e.file_type().is_some_and(|t| t.is_file())
}

fn has_terraform_extension(e: &DirEntry) -> bool {
    let file_name: PathBuf = e.file_name().into();
    let extension = file_name.extension();
    extension.is_some_and(|ext| ext == "tf")
}

fn is_dir_or_terraform_file(e: &DirEntry) -> bool {
    !is_file(e) || has_terraform_extension(e)
}

pub fn root<P>(path: P) -> impl Iterator<Item = DirEntry>
where
    P: AsRef<Path>,
{
    WalkBuilder::new(path)
        .filter_entry(is_dir_or_terraform_file)
        .build()
        .filter_map(std::result::Result::ok)
}

pub fn find_files<P>(path: P) -> impl Iterator<Item = PathBuf>
where
    P: AsRef<Path>,
{
    root(path).filter_map(|e| {
        if is_file(&e) {
            Some(e.path().to_owned())
        } else {
            None
        }
    })
}

pub fn find_roots<P>(path: P) -> impl Iterator<Item = PathBuf>
where
    P: AsRef<Path>,
{
    root(path).filter_map(|e| {
        if is_top_level(e.path()) {
            e.path().parent().map(std::borrow::ToOwned::to_owned)
        } else {
            None
        }
    })
}

fn string_pairs_key_prepend(
    prefix: &str,
    string_pairs: Vec<(String, String)>,
) -> Vec<(String, String)> {
    let mut ret = Vec::new();
    for (k, v) in string_pairs {
        ret.push((format!("{prefix}:{k}"), v));
    }
    ret
}

fn string_repititions_accross_roots(
    roots: impl Iterator<Item = PathBuf>,
    min_repetitions: usize,
) -> IndexMap<String, HashSet<String>> {
    let mut ret = IndexMap::<String, HashSet<String>>::new();

    for root in roots {
        for file in find_files(&root) {
            if let Ok(body) = parse::<hcl::Body, _>(&file) {
                for (k, v) in string_pairs_key_prepend(root.to_str().unwrap_or(""), strings(&body))
                {
                    if let Some(addresses) = ret.get_mut(&v) {
                        addresses.insert(k);
                    } else {
                        ret.insert(v, [k].into());
                    }
                }
            }
        }
    }

    ret.sort_by(|_, v1, _, v2| v2.len().cmp(&v1.len()));

    // TODO something less O(N)-y
    let mut prune_by_repetition_count_from: Option<usize> = None;
    let mut idx = 0;
    for v in ret.values() {
        if v.len() < min_repetitions {
            prune_by_repetition_count_from = Some(idx);
            break;
        }
        idx += 1;
    }
    if let Some(idx) = prune_by_repetition_count_from {
        ret.truncate(idx);
    }

    ret
}

pub fn string_repetitions<P>(path: P, min_repetitions: usize) -> IndexMap<String, HashSet<String>>
where
    P: AsRef<Path>,
{
    let mut roots = find_roots(&path).peekable();
    if roots.peek().is_none() {
        return string_repititions_accross_roots(once(path.as_ref().to_owned()), min_repetitions);
    }
    string_repititions_accross_roots(roots, min_repetitions)
}

#[cfg(test)]
mod test {
    use super::*;
    use test_files::TestFiles;

    macro_rules! string_reps {
            ($root:expr; $($k:expr => $vals:tt),* $(,)?) => {{
                [$(($k.to_string(), string_reps!(@vals $root; $vals))),*].into()
            }};
            (@vals $root:expr; [$($v:expr),+ $(,)?]) => {{
                [$(format!("{}:{}", $root.display(), $v)),*].into()
            }};
        }

    #[test]
    fn basic_string_repetitions_no_terraform_roots() {
        let temp_dir = TestFiles::new();
        temp_dir
            .file(
                "foo.tf",
                r#"
                resource thing "s" {
                    were = "weird"
                    got = "wild"
                }
                "#,
            )
            .file(
                "bar.tf",
                r#"
                module stuff {
                    in = {
                        the = "wild"
                        sanely = "repetitive"
                    }
                    gets = "repetitive"
                    is = "repetitive"
                }
            "#,
            );

        assert_eq!(
            string_repetitions(temp_dir.path(), 0),
            string_reps! {
                temp_dir.path();
                "repetitive" => [
                    "module.stuff.in.sanely",
                    "module.stuff.gets",
                    "module.stuff.is",
                ],
                "wild" => [
                    "resource.thing.s.got",
                    "module.stuff.in.the",
                ],
                "weird" => [
                    "resource.thing.s.were",
                ],
            }
        );

        assert_eq!(
            string_repetitions(temp_dir.path(), 1),
            string_reps! {
                temp_dir.path();
                "repetitive" => [
                    "module.stuff.in.sanely",
                    "module.stuff.gets",
                    "module.stuff.is",
                ],
                "wild" => [
                    "resource.thing.s.got",
                    "module.stuff.in.the",
                ],
                "weird" => [
                    "resource.thing.s.were",
                ],
            }
        );

        assert_eq!(
            string_repetitions(temp_dir.path(), 2),
            string_reps! {
                temp_dir.path();
                "repetitive" => [
                    "module.stuff.in.sanely",
                    "module.stuff.gets",
                    "module.stuff.is",
                ],
                "wild" => [
                    "resource.thing.s.got",
                    "module.stuff.in.the",
                ],
            }
        );

        assert_eq!(
            string_repetitions(temp_dir.path(), 3),
            string_reps! {
                temp_dir.path();
                "repetitive" => [
                    "module.stuff.in.sanely",
                    "module.stuff.gets",
                    "module.stuff.is",
                ],
            }
        )
    }

    #[test]
    fn basic_string_repetition_multiple_terraform_roots() {
        let temp_dir = TestFiles::new();
        temp_dir
            .file(
                "london/terraform.tf",
                r#"
                terraform {
                    backend "s3" {
                        bucket         = "eu-west-2"
                    }
                }
                "#,
            )
            .file(
                "london/foo.tf",
                r#"
                resource thing "s" {
                    were = "weird"
                    got = "wild"
                }
                "#,
            )
            .file(
                "london/bar.tf",
                r#"
                module stuff {
                    in = {
                        the = "wild"
                        sanely = "repetitive"
                    }
                    gets = "repetitive"
                    is = "repetitive"
                }
            "#,
            )
            .file(
                "tokyo/terraform.tf",
                r#"
                terraform {
                    backend "s3" {
                        bucket         = "ap-northeast-1"
                    }
                }
                "#,
            )
            .file(
                "tokyo/foo.tf",
                r#"
                resource thing "s" {
                    were = "weird"
                    got = "wild"
                }
                "#,
            )
            .file(
                "tokyo/bar.tf",
                r#"
                module stuff {
                    in = {
                        the = "wild"
                        sanely = "repetitive"
                    }
                    gets = "repetitive"
                    is = "repetitive"
                }
            "#,
            );

        assert_eq!(
            string_repetitions(temp_dir.path(), 2),
            string_reps! {
                temp_dir.path();
                "repetitive" => [
                    "london:module.stuff.in.sanely",
                    "london:module.stuff.gets",
                    "london:module.stuff.is",
                    "tokyo:module.stuff.in.sanely",
                    "tokyo:module.stuff.gets",
                    "tokyo:module.stuff.is",
                ],
                "wild" => [
                    "london:resource.thing.s.got",
                    "london:module.stuff.in.the",
                    "tokyo:resource.thing.s.got",
                    "tokyo:module.stuff.in.the",
                ],
                "weird" => [
                    "london:resource.thing.s.were",
                    "tokyo:resource.thing.s.were",
                ],
            }
        )
    }
}
