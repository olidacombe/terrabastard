use std::io::Read;

use hcl::Block;

use self::json::PolicyDocument;

pub mod json;

pub fn json_iam_policy_to_data_resource<R: Read, S: AsRef<str>>(
    json: R,
    name: S,
) -> Result<Block, std::io::Error> {
    let json_policy: PolicyDocument = serde_json::from_reader(json)?;
    let hcl_policy = json_policy.to_hcl(name.as_ref());
    Ok(hcl_policy)
}
