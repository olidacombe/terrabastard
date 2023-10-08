use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize)]
pub enum PolicyVersion {
    #[serde(rename = "2012-10-17")]
    V20121017,
}

#[derive(Deserialize)]
pub enum Effect {
    Allow,
    Deny,
}

// TODO `NotAction`
#[derive(Deserialize)]
#[serde(untagged)]
pub enum Action {
    Mono(String),
    Poly(Vec<String>),
}

#[derive(Deserialize, PartialEq, Eq, Hash)]
pub enum ConditionOperator {
    NumericLessThanEquals,
    StringLike,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum ConditionValue {
    Mono(String),
    Poly(Vec<String>),
}

#[derive(Deserialize)]
pub struct ConditionOperands(HashMap<String, ConditionValue>);

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PolicyStatement {
    sid: String,
    effect: Effect,
    resource: String,
    condition: HashMap<ConditionOperator, ConditionOperands>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Statement {
    Mono(PolicyStatement),
    Poly(Vec<PolicyStatement>),
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct JsonPolicyDocument {
    version: PolicyVersion,
    statement: PolicyStatement,
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

        let _: JsonPolicyDocument = serde_json::from_str(data)?;

        Ok(())
    }
}
