use std::collections::HashMap;

use hcl::Block;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Deserialize, Display, Serialize)]
pub enum PolicyVersion {
    #[serde(rename = "2008-10-17")]
    #[strum(serialize = "2008-10-17")]
    V20081017,
    #[serde(rename = "2012-10-17")]
    #[strum(serialize = "2012-10-17")]
    V20121017,
}

#[derive(Clone, Deserialize, Display)]
pub enum Effect {
    Allow,
    Deny,
}

#[derive(Clone, Deserialize, Display, PartialEq, Eq, Hash)]
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
                for (variable, values) in operands.0 {
                    let values: Vec<String> = values.into_iter().collect();
                    builder = builder.add_block(
                        Block::builder("condition")
                            .add_attribute(("test", operator.to_string()))
                            .add_attribute(("variable", variable))
                            .add_attribute(("values", values))
                            .build(),
                    );
                }
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
            .add_label(name)
            .add_attribute(("version", format!("{}", &self.version)));
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

        insta::assert_snapshot!(hcl::to_string(&hcl_policy)?);

        Ok(())
    }
}
