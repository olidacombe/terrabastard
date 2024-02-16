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
    ArnEquals,
    ArnLike,
    ArnNotEquals,
    ArnNotLike,
    BinaryEquals,
    Bool,
    DateEquals,
    DateGreaterThan,
    DateGreaterThanEquals,
    DateLessThan,
    DateLessThanEquals,
    DateNotEquals,
    IfExists,
    IpAddress,
    NotIpAddress,
    Null,
    NumericEquals,
    NumericGreaterThan,
    NumericGreaterThanEquals,
    NumericLessThan,
    NumericLessThanEquals,
    NumericNotEquals,
    StringEquals,
    StringEqualsIgnoreCase,
    StringLike,
    StringLikeIfExists,
    StringNotEquals,
    StringNotEqualsIgnoreCase,
    StringNotLike,
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

#[derive(Clone, Deserialize, Display, PartialEq, Eq, Hash)]
pub enum Principal {
    AWS,
    CanonicalUser,
    Federated,
    Service,
    // TODO
    // Star, // "*"
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Statement {
    sid: Option<String>,
    effect: Effect,
    action: OneOrMany<String>,
    resource: Option<OneOrMany<String>>,
    condition: Option<HashMap<ConditionOperator, ConditionOperands>>,
    principal: Option<HashMap<Principal, OneOrMany<String>>>,
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
        if let Some(resources) = statement.resource {
            builder = builder
                .add_attribute(("resources", resources.into_iter().collect::<Vec<String>>()));
        }
        if let Some(principals) = statement.principal {
            for (ty, identifiers) in principals {
                let identifiers: Vec<String> = identifiers.into_iter().collect();
                builder = builder.add_block(
                    Block::builder("principals")
                        .add_attribute(("type", ty.to_string()))
                        .add_attribute(("identifiers", identifiers))
                        .build(),
                );
            }
        }
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
    fn example_1() -> Result<()> {
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

    #[test]
    fn example_2() -> Result<()> {
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

        let json_policy: PolicyDocument = serde_json::from_str(data)?;
        let hcl_policy = json_policy.to_hcl("example_2");

        insta::assert_snapshot!(hcl::to_string(&hcl_policy)?);

        Ok(())
    }

    #[test]
    fn example_3() -> Result<()> {
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
        let hcl_policy = json_policy.to_hcl("example_3");

        insta::assert_snapshot!(hcl::to_string(&hcl_policy)?);

        Ok(())
    }

    #[test]
    fn example_4() -> Result<()> {
        let data = r#"                                                                                          
        {                                                                                                   
            "Version": "2012-10-17",                                                                        
            "Statement": [                                                                                  
                {                                                                                           
                    "Sid": "ec2",                                                                           
                    "Effect": "Allow",                                                                      
                    "Principal": {                                                                          
                      "Service": "ec2.amazonaws.com"                                                        
                    },                                                                                      
                    "Action": "sts:AssumeRole"                                                              
                },                                                                                          
                {                                                                                           
                    "Sid": "zebra",                                                                         
                    "Effect": "Allow",                                                                      
                    "Principal": {                                                                          
                        "AWS": [                                                                            
                            "arn:aws:iam::bar:role/zebra-agents",                                           
                            "arn:aws:iam::bar:role/zebra-master"                                            
                        ]                                                                                   
                    },                                                                                      
                    "Action": "sts:AssumeRole"                                                              
                },                                                                                          
                {                                                                                           
                    "Effect": "Allow",                                                                      
                    "Principal": {                                                                          
                        "Federated": [                                                                      
                            "arn:aws:iam::baz:oidc-provider/oidc.eks.eu-central-1.amazonaws.com/id/foo"     
                        ]                                                                                   
                    },                                                                                      
                    "Action": "sts:AssumeRoleWithWebIdentity",                                              
                    "Condition": {                                                                          
                        "StringLikeIfExists": {                                                             
                            "oidc.eks.eu-central-1.amazonaws.com/id/foo:*": "*"                             
                        }                                                                                   
                    }                                                                                       
                }                                                                                           
            ]                                                                                               
        }"#;

        let json_policy: PolicyDocument = serde_json::from_str(data)?;
        let hcl_policy = json_policy.to_hcl("example_4");

        insta::assert_snapshot!(hcl::to_string(&hcl_policy)?);

        Ok(())
    }
}
