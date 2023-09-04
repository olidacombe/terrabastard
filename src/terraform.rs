use eyre::Result;
use hcl::{Attribute, Block, BlockLabel, Expression, Object, ObjectKey, Structure};
use serde::{de::DeserializeOwned, Deserialize};
use std::{fs::File, path::Path};

#[derive(Deserialize)]
pub struct S3BackendConfig {
    pub bucket: String,
}

#[derive(Deserialize)]
pub enum BackendConfig {
    #[serde(rename = "s3")]
    S3(S3BackendConfig),
}

#[allow(clippy::module_name_repetitions)]
#[derive(Deserialize)]
pub struct TerraformBlock {
    pub backend: BackendConfig,
}

#[derive(Deserialize)]
pub struct TopLevel {
    pub terraform: TerraformBlock,
}

impl TopLevel {
    pub fn parse<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        parse(path)
    }
}

pub fn parse<T, P>(file_path: P) -> Result<T>
where
    T: DeserializeOwned,
    P: AsRef<Path>,
{
    let f = File::open(file_path)?;
    Ok(hcl::from_reader(f)?)
}

pub fn is_top_level<P>(path: P) -> bool
where
    P: AsRef<Path>,
{
    TopLevel::parse(path).is_ok()
}

fn object_strings(object: &Object<ObjectKey, Expression>) -> Vec<(String, String)> {
    let mut ret = Vec::new();
    for (k, v) in object {
        // only handle the basics for now
        if let ObjectKey::Identifier(i) = k {
            match v {
                Expression::String(s) => {
                    ret.push((i.to_string(), s.clone()));
                }
                Expression::Object(o) => {
                    ret.append(&mut string_pairs_key_prepend(i.as_str(), object_strings(o)));
                }
                _ => {}
            }
        }
    }
    ret
}

fn string_pairs_key_prepend(
    prefix: &str,
    string_pairs: Vec<(String, String)>,
) -> Vec<(String, String)> {
    let mut ret = Vec::new();
    for (k, v) in string_pairs {
        ret.push((format!("{prefix}.{k}"), v));
    }
    ret
}

pub fn strings(body: &hcl::Body) -> Vec<(String, String)> {
    let mut ret = Vec::new();
    for structure in &body.0 {
        match structure {
            Structure::Attribute(Attribute { key, expr }) => match expr {
                Expression::String(s) => {
                    ret.push((key.as_str().to_string(), s.clone()));
                }
                Expression::Object(o) => {
                    ret.append(&mut string_pairs_key_prepend(
                        key.as_str(),
                        object_strings(&o),
                    ));
                }
                _ => {}
            },
            Structure::Block(Block {
                identifier,
                labels,
                body,
            }) => {
                let prefix = if labels.is_empty() {
                    identifier.as_str().to_string()
                } else {
                    format!(
                        "{}.{}",
                        identifier.as_str(),
                        labels
                            .iter()
                            .map(|l| match l {
                                BlockLabel::Identifier(i) => i.as_str(),
                                BlockLabel::String(s) => s.as_str(),
                            })
                            .collect::<Vec<&str>>()
                            .join(".")
                    )
                };
                for (k, v) in strings(body) {
                    ret.push((format!("{prefix}.{k}"), v));
                }
            }
        }
    }
    ret
}

#[cfg(test)]
mod test {
    use super::*;
    use eyre::Result;
    use test_files::TestFiles;

    #[test]
    fn parse_deserializes_a_terraform_root() -> Result<()> {
        let temp_dir = TestFiles::new();
        temp_dir
            .file(
                "foo.tf",
                r#"
                terraform {
                    backend "s3" {
                        bucket         = "bucky"
                        dynamodb_table = "terraform-state-lock"
                        region         = "eu-central-1"
                        key            = "my/state"
                        encrypt        = true
                    }
                }
                "#,
            )
            .file(
                "bar.tf",
                r#"
                terraform {
                    backend "s3" {
                        smucket         = "bucky"
                    }
                }
            "#,
            );

        // not a top-level terraform file
        let tf: Result<TopLevel> = parse(temp_dir.path().join("bar.tf"));
        assert!(tf.is_err());
        // legit top-level terraform file
        let _tf: TopLevel = parse(temp_dir.path().join("foo.tf"))?;
        Ok(())
    }

    #[test]
    fn is_top_level_terraform_works() {
        let temp_dir = TestFiles::new();
        temp_dir
            .file(
                "foo.tf",
                r#"
                terraform {
                    backend "s3" {
                        bucket         = "bucky"
                        dynamodb_table = "terraform-state-lock"
                        region         = "eu-central-1"
                        key            = "my/state"
                        encrypt        = true
                    }
                }
                "#,
            )
            .file(
                "bar.tf",
                r#"
                terraform {
                    backend "s3" {
                        smucket         = "bucky"
                    }
                }
            "#,
            );

        assert!(is_top_level(temp_dir.path().join("foo.tf")));
        assert!(!is_top_level(temp_dir.path().join("bar.tf")));
    }

    fn expr_as_obj(expr: Expression) -> Object<ObjectKey, Expression> {
        if let Expression::Object(obj) = expr {
            return obj;
        }
        panic! {"Bad test construction, expression should be Object variant {:?}", expr};
    }

    #[test]
    fn strings_uses_deep_object_addresses() {
        let obj = expr_as_obj(hcl::expression!({ foo = { bar = "baz" } }));
        assert_eq!(
            object_strings(&obj),
            [("foo.bar".to_string(), "baz".to_string())]
        );
    }
}
