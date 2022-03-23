use cosmwasm_std::HumanAddr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Jackpot {
  pub word: String,
  pub amount: u64,
  pub complexity_min: u32,
  pub complexity_max: u32,
  pub shown: Vec<HumanAddr>,
  pub first_claim: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
  pub jackpots: Vec<Jackpot>,
  pub nft_contract: HumanAddr,
  pub nft_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
  Fund {},
  LiberateFunds {
    target: HumanAddr,
  },
  UpdateComplexity {
    min: u32,
    max: u32,
    index: u8,
  },
  NextWord {
    index: u8,
  },
  ShowMeTheMoney {
    jackpot_index: u8,
    nft_id: String,
    viewing_key: String,
  },
}

/// the address and viewing key making an authenticated query request
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ViewerInfo {
  /// querying address
  pub address: HumanAddr,
  /// authentication key string
  pub viewing_key: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NFTQueries {
  PrivateMetadata {
    token_id: String,
    /// optional address and key requesting to view the private metadata
    viewer: Option<ViewerInfo>,
  },
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Debug, Default)]
pub struct Trait {
  /// indicates how a trait should be displayed
  pub display_type: Option<String>,
  /// name of the trait
  pub trait_type: Option<String>,
  /// trait value
  pub value: String,
  /// optional max value for numerical traits
  pub max_value: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Debug, Default)]
pub struct Authentication {
  /// either a decryption key for encrypted files or a password for basic authentication
  pub key: Option<String>,
  /// username used in basic authentication
  pub user: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Debug, Default)]
pub struct MediaFile {
  /// file type
  /// Stashh currently uses: "image", "video", "audio", "text", "font", "application"
  pub file_type: Option<String>,
  /// file extension
  pub extension: Option<String>,
  /// authentication information
  pub authentication: Option<Authentication>,
  /// url to the file.  Urls should be prefixed with `http://`, `https://`, `ipfs://`, or `ar://`
  pub url: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Debug, Default)]
pub struct Extension {
  /// url to the image
  pub image: Option<String>,
  /// raw SVG image data (not recommended). Only use this if you're not including the image parameter
  pub image_data: Option<String>,
  /// url to allow users to view the item on your site
  pub external_url: Option<String>,
  /// item description
  pub description: Option<String>,
  /// name of the item
  pub name: Option<String>,
  /// item attributes
  pub attributes: Option<Vec<Trait>>,
  /// background color represented as a six-character hexadecimal without a pre-pended #
  pub background_color: Option<String>,
  /// url to a multimedia attachment
  pub animation_url: Option<String>,
  /// url to a YouTube video
  pub youtube_url: Option<String>,
  /// media files as specified on Stashh that allows for basic authenticatiion and decryption keys.
  /// Most of the above is used for bridging public eth NFT metadata easily, whereas `media` will be used
  /// when minting NFTs on Stashh
  pub media: Option<Vec<MediaFile>>,
  /// a select list of trait_types that are in the private metadata.  This will only ever be used
  /// in public metadata
  pub protected_attributes: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum NFTQueryAnswers {
  PrivateMetadata {
    token_uri: Option<String>,
    extension: Option<Extension>,
  },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
  GetJackpots {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct JackpotResponse {
  pub word: String,
  pub amount: u64,
  pub claimants: Vec<HumanAddr>,
  pub claimable_time: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct JackpotsResponse {
  pub jackpots: Vec<JackpotResponse>,
}
