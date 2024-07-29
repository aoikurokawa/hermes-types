use std::collections::BTreeMap;

use base64::{engine::general_purpose::STANDARD as base64_standard_engine, Engine as _};
use borsh::{BorshDeserialize, BorshSerialize};
use derive_more::{Deref, DerefMut};
use pyth_sdk::{Price, PriceFeed, PriceIdentifier};
use serde::{Deserialize, Serialize};
use wormhole_sdk::Chain;

mod serde_hex;

pub type Slot = u64;

/// The number of seconds since the Unix epoch (00:00:00 UTC on 1 Jan 1970). The timestamp is
/// always positive, but represented as a signed integer because that's the standard on Unix
/// systems and allows easy subtraction to compute durations.
pub type UnixTimestamp = i64;

#[derive(Debug, PartialEq)]
pub struct PriceFeedUpdate {
    pub price_feed: PriceFeed,
    pub slot: Option<Slot>,
    pub received_at: Option<UnixTimestamp>,
    pub update_data: Option<Vec<u8>>,
    pub prev_publish_time: Option<UnixTimestamp>,
}

#[derive(Debug, PartialEq)]
pub struct PriceFeedsWithUpdateData {
    pub price_feeds: Vec<PriceFeedUpdate>,
    pub update_data: Vec<Vec<u8>>,
}

/// A price id is a 32-byte hex string, optionally prefixed with "0x".
/// Price ids are case insensitive.
///
/// Examples:
/// * 0xe62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43
/// * e62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43
///
/// See https://pyth.network/developers/price-feed-ids for a list of all price feed ids.
#[derive(Clone, Debug, Deref, DerefMut, Deserialize, Serialize)]
pub struct PriceIdInput(#[serde(with = "crate::serde_hex")] [u8; 32]);

impl From<PriceIdInput> for PriceIdentifier {
    fn from(id: PriceIdInput) -> Self {
        Self::new(*id)
    }
}

type Base64String = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcPriceFeedMetadata {
    pub slot: Option<Slot>,
    pub emitter_chain: u16,
    pub price_service_receive_time: Option<UnixTimestamp>,
    pub prev_publish_time: Option<UnixTimestamp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcPriceFeedMetadataV2 {
    pub slot: Option<Slot>,
    pub proof_available_time: Option<UnixTimestamp>,
    pub prev_publish_time: Option<UnixTimestamp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcPriceFeed {
    pub id: RpcPriceIdentifier,
    pub price: RpcPrice,
    pub ema_price: RpcPrice,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<RpcPriceFeedMetadata>,

    /// The VAA binary represented as a base64 string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vaa: Option<Base64String>,
}

impl RpcPriceFeed {
    // TODO: Use a Encoding type to have None, Base64, and Hex variants instead of binary flag.
    // TODO: Use a Verbosity type to define None, or Full instead of verbose flag.
    pub fn from_price_feed_update(
        price_feed_update: PriceFeedUpdate,
        verbose: bool,
        binary: bool,
    ) -> Self {
        let price_feed = price_feed_update.price_feed;

        Self {
            id: RpcPriceIdentifier::new(price_feed.id.to_bytes()),
            price: RpcPrice {
                price: price_feed.get_price_unchecked().price,
                conf: price_feed.get_price_unchecked().conf,
                expo: price_feed.get_price_unchecked().expo,
                publish_time: price_feed.get_price_unchecked().publish_time,
            },
            ema_price: RpcPrice {
                price: price_feed.get_ema_price_unchecked().price,
                conf: price_feed.get_ema_price_unchecked().conf,
                expo: price_feed.get_ema_price_unchecked().expo,
                publish_time: price_feed.get_ema_price_unchecked().publish_time,
            },
            metadata: verbose.then_some(RpcPriceFeedMetadata {
                emitter_chain: Chain::Pythnet.into(),
                price_service_receive_time: price_feed_update.received_at,
                slot: price_feed_update.slot,
                prev_publish_time: price_feed_update.prev_publish_time,
            }),
            vaa: match binary {
                false => None,
                true => price_feed_update
                    .update_data
                    .map(|data| base64_standard_engine.encode(data)),
            },
        }
    }
}

/// A price with a degree of uncertainty at a certain time, represented as a price +- a confidence
/// interval.
///
/// The confidence interval roughly corresponds to the standard error of a normal distribution.
/// Both the price and confidence are stored in a fixed-point numeric representation, `x *
/// 10^expo`, where `expo` is the exponent. For example:
#[derive(
    Clone,
    Copy,
    Default,
    Debug,
    PartialEq,
    Eq,
    BorshSerialize,
    BorshDeserialize,
    Serialize,
    Deserialize,
)]
pub struct RpcPrice {
    /// The price itself, stored as a string to avoid precision loss
    #[serde(with = "pyth_sdk::utils::as_string")]
    pub price: i64,

    /// The confidence interval associated with the price, stored as a string to avoid precision loss
    #[serde(with = "pyth_sdk::utils::as_string")]
    pub conf: u64,

    /// The exponent associated with both the price and confidence interval. Multiply those values
    /// by `10^expo` to get the real value.
    pub expo: i32,

    /// When the price was published. The `publish_time` is a unix timestamp, i.e., the number of
    /// seconds since the Unix epoch (00:00:00 UTC on 1 Jan 1970).
    pub publish_time: UnixTimestamp,
}

#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    BorshSerialize,
    BorshDeserialize,
    Serialize,
    Deserialize,
)]
#[repr(C)]
pub struct RpcPriceIdentifier(#[serde(with = "hex")] [u8; 32]);

impl RpcPriceIdentifier {
    pub fn new(bytes: [u8; 32]) -> RpcPriceIdentifier {
        RpcPriceIdentifier(bytes)
    }
}

impl From<RpcPriceIdentifier> for PriceIdentifier {
    fn from(id: RpcPriceIdentifier) -> Self {
        PriceIdentifier::new(id.0)
    }
}

impl From<PriceIdentifier> for RpcPriceIdentifier {
    fn from(id: PriceIdentifier) -> Self {
        RpcPriceIdentifier(id.to_bytes())
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub enum EncodingType {
    #[default]
    #[serde(rename = "hex")]
    Hex,
    #[serde(rename = "base64")]
    Base64,
}

impl EncodingType {
    pub fn encode_str(&self, data: &[u8]) -> String {
        match self {
            EncodingType::Base64 => base64_standard_engine.encode(data),
            EncodingType::Hex => hex::encode(data),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryPriceUpdate {
    pub encoding: EncodingType,
    pub data: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedPriceUpdate {
    pub id: RpcPriceIdentifier,
    pub price: RpcPrice,
    pub ema_price: RpcPrice,
    pub metadata: RpcPriceFeedMetadataV2,
}

impl From<PriceFeedUpdate> for ParsedPriceUpdate {
    fn from(price_feed_update: PriceFeedUpdate) -> Self {
        let price_feed = price_feed_update.price_feed;

        Self {
            id: RpcPriceIdentifier::from(price_feed.id),
            price: RpcPrice {
                price: price_feed.get_price_unchecked().price,
                conf: price_feed.get_price_unchecked().conf,
                expo: price_feed.get_price_unchecked().expo,
                publish_time: price_feed.get_price_unchecked().publish_time,
            },
            ema_price: RpcPrice {
                price: price_feed.get_ema_price_unchecked().price,
                conf: price_feed.get_ema_price_unchecked().conf,
                expo: price_feed.get_ema_price_unchecked().expo,
                publish_time: price_feed.get_ema_price_unchecked().publish_time,
            },
            metadata: RpcPriceFeedMetadataV2 {
                proof_available_time: price_feed_update.received_at,
                slot: price_feed_update.slot,
                prev_publish_time: price_feed_update.prev_publish_time,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceUpdate {
    pub binary: BinaryPriceUpdate,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parsed: Option<Vec<ParsedPriceUpdate>>,
}

impl TryFrom<PriceUpdate> for PriceFeedsWithUpdateData {
    type Error = anyhow::Error;

    fn try_from(price_update: PriceUpdate) -> anyhow::Result<Self> {
        let price_feeds = match price_update.parsed {
            Some(parsed_updates) => parsed_updates
                .into_iter()
                .map(|parsed_price_update| {
                    Ok(PriceFeedUpdate {
                        price_feed: PriceFeed::new(
                            parsed_price_update.id.into(),
                            Price {
                                price: parsed_price_update.price.price,
                                conf: parsed_price_update.price.conf,
                                expo: parsed_price_update.price.expo,
                                publish_time: parsed_price_update.price.publish_time,
                            },
                            Price {
                                price: parsed_price_update.ema_price.price,
                                conf: parsed_price_update.ema_price.conf,
                                expo: parsed_price_update.ema_price.expo,
                                publish_time: parsed_price_update.ema_price.publish_time,
                            },
                        ),
                        slot: parsed_price_update.metadata.slot,
                        received_at: parsed_price_update.metadata.proof_available_time,
                        update_data: None, // This field is not available in ParsedPriceUpdate
                        prev_publish_time: parsed_price_update.metadata.prev_publish_time,
                    })
                })
                .collect::<anyhow::Result<Vec<_>>>(),
            None => Err(anyhow::anyhow!("No parsed price updates available")),
        }?;

        let update_data = price_update
            .binary
            .data
            .iter()
            .map(|hex_str| hex::decode(hex_str).unwrap_or_default())
            .collect::<Vec<Vec<u8>>>();

        Ok(PriceFeedsWithUpdateData {
            price_feeds,
            update_data,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceFeedMetadata {
    pub id: RpcPriceIdentifier,
    // BTreeMap is used to automatically sort the keys to ensure consistent ordering of attributes in the JSON response.
    // This enhances user experience by providing a predictable structure, avoiding confusion from varying orders in different responses.
    pub attributes: BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AssetType {
    Crypto,
    FX,
    Equity,
    Metals,
    Rates,
}

impl std::fmt::Display for AssetType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
