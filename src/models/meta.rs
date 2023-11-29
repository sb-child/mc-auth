pub mod meta_resp {
  use serde::{Deserialize, Serialize};

  #[derive(Serialize, Deserialize, Debug, Clone)]
  pub struct MetaLinks {
    pub homepage: String,
    pub register: String,
  }

  #[derive(Serialize, Deserialize, Debug, Clone)]
  pub struct Meta {
    #[serde(rename = "serverName")]
    pub server_name: String,
    #[serde(rename = "implementationName")]
    pub implementation_name: String,
    #[serde(rename = "implementationVersion")]
    pub implementation_version: String,
    pub links: MetaLinks,
  }

  #[derive(Serialize, Deserialize, Debug, Clone)]
  pub struct GetMetadataResp {
    pub meta: Meta,
    #[serde(rename = "skinDomains")]
    pub skin_domains: Vec<String>,
    #[serde(rename = "signaturePublickey")]
    pub signature_publickey: String,
  }
}
