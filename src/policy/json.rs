use std::collections::HashMap;

use hcl::Block;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Deserialize)]
pub enum PolicyVersion {
    #[serde(rename = "2008-10-17")]
    V20081017,
    #[serde(rename = "2012-10-17")]
    V20121017,
}

#[derive(Clone, Deserialize, Display)]
pub enum Effect {
    Allow,
    Deny,
}

#[derive(Clone, Deserialize, PartialEq, Eq, Hash)]
pub enum ConditionOperator {
    NumericLessThanEquals,
    StringLike,
}

#[derive(Deserialize, Clone, Serialize)]
#[serde(untagged)]
pub enum OneOrMany<T>
where
    T: Clone,
{
    Mono(T),
    Poly(Vec<T>),
}

impl<T: Clone> IntoIterator for OneOrMany<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let vec = match self {
            OneOrMany::Mono(t) => vec![t],
            OneOrMany::Poly(v) => v,
        };
        vec.into_iter()
    }
}

#[derive(Clone, Deserialize)]
pub struct ConditionOperands(HashMap<String, OneOrMany<String>>);

#[derive(Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Statement {
    sid: Option<String>,
    effect: Effect,
    action: OneOrMany<String>,
    resource: OneOrMany<String>,
    condition: Option<HashMap<ConditionOperator, ConditionOperands>>,
}

impl From<Statement> for Block {
    fn from(statement: Statement) -> Self {
        let actions: Vec<String> = statement.action.into_iter().collect();
        let mut builder = Block::builder("statement")
            .add_attribute(("effect", statement.effect.to_string()))
            .add_attribute(("actions", actions));
        if let Some(sid) = statement.sid {
            builder = builder.add_attribute(("sid", sid));
        }
        builder = builder.add_attribute((
            "resources",
            statement.resource.into_iter().collect::<Vec<String>>(),
        ));
        if let Some(condition) = statement.condition {
            for (operator, operands) in condition {
                // TODO pick up here
                builder = builder.add_block(Block::builder("condition").build());
            }
        }
        builder.build()
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PolicyDocument {
    version: PolicyVersion,
    statement: OneOrMany<Statement>,
}

impl PolicyDocument {
    pub fn to_hcl(&self, name: &str) -> Block {
        let mut builder = Block::builder("resource")
            .add_label("aws_iam_policy_document")
            .add_label(name);
        for statement in self.statement.clone() {
            builder = builder.add_block(Block::from(statement));
        }
        builder.build()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use eyre::Result;

    #[test]
    fn deserialize_example_1() -> Result<()> {
        let data = r#"{
            "Version": "2012-10-17",
            "Statement": {
                "Sid": "AllowRemoveMfaOnlyIfRecentMfa",
                "Effect": "Allow",
                "Action": [
                    "iam:DeactivateMFADevice"
                ],
                "Resource": "arn:aws:iam::*:user/${aws:username}",
                "Condition": {
                    "NumericLessThanEquals": {"aws:MultiFactorAuthAge": "3600"}
                }
            }
        }"#;

        let _: PolicyDocument = serde_json::from_str(data)?;

        Ok(())
    }

    #[test]
    fn deserialize_example_2() -> Result<()> {
        let data = r#"{
          "Version": "2012-10-17",
          "Statement": [
            {
              "Action": [
                "ec2:DescribeInstances",
                "ec2:DescribeSecurityGroups",
                "ec2:DescribeSecurityGroupReferences",
                "ec2:DescribeStaleSecurityGroups"
              ],
              "Resource": "*",
              "Effect": "Allow"
            },
            {
              "Action": [
                "ec2:AuthorizeSecurityGroupEgress",
                "ec2:AuthorizeSecurityGroupIngress",
                "ec2:RevokeSecurityGroupEgress",
                "ec2:RevokeSecurityGroupIngress",
                "ec2:StartInstances",
                "ec2:StopInstances"
              ],
              "Resource": [
                "arn:aws:ec2:*:*:instance/i-instance-id",
                "arn:aws:ec2:*:*:security-group/sg-security-group-id"
              ],
              "Effect": "Allow"
            }
          ]
        }"#;

        let _: PolicyDocument = serde_json::from_str(data)?;

        Ok(())
    }

    #[test]
    fn to_hcl_example_1() -> Result<()> {
        let data = r#"{
            "Version": "2012-10-17",
            "Statement": {
                "Sid": "AllowRemoveMfaOnlyIfRecentMfa",
                "Effect": "Allow",
                "Action": [
                    "iam:DeactivateMFADevice"
                ],
                "Resource": "arn:aws:iam::*:user/${aws:username}",
                "Condition": {
                    "NumericLessThanEquals": {"aws:MultiFactorAuthAge": "3600"}
                }
            }
        }"#;

        let json_policy: PolicyDocument = serde_json::from_str(data)?;
        let hcl_policy = json_policy.to_hcl("example_1");

        let expected = r#"
            resource "aws_iam_policy_document" "example_1" {
                statement {
                    sid = "AllowRemoveMfaOnlyIfRecentMfa"
                    effect = "Allow"
                    actions = ["iam:DeactivateMFADevice"]
                    resources = ["arn:aws:iam::*:user/${aws:username}"]
                    condition {
                        test = "NumericLessThanEquals"
                        variable = "aws:MultiFactorAuthAge"
                        values = ["3600"]
                    }
                }
            }
        "#
        .trim_start();

        insta::assert_snapshot!(hcl::to_string(&hcl_policy)?);

        Ok(())
    }
}
