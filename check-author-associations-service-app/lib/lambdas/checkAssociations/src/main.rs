use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use lambda_runtime::handler_fn;
use std::collections::HashMap;
use futures_util::future::join_all;

#[derive(Deserialize)]
pub struct Request {
    pub _body: String,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub body: String,
}

#[derive(Debug, Serialize)]
pub struct FailureResponse {
    pub body: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserNftsRequest {
    #[serde(rename = "UserPublicKeyBase58Check")]
    pub user_public_key: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NFTs {
    #[serde(rename = "NFTsMap")]
    pub nfts: HashMap<String, NFTData>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NFTData {
    #[serde(rename = "NFTEntryResponses")]
    pub nft_responses: Vec<NFTEntry>,
    #[serde(rename = "PostEntryResponse")]
    pub post: PostEntryResponse
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NFTEntry {
    #[serde(rename = "OwnerPublicKeyBase58Check")]
    pub owner: String, // Public key of the user who owns this serial number
    #[serde(rename = "SerialNumber")]
    pub serial_number: u32, // serial number described by this NFTEntryResponse
    #[serde(rename = "IsForSale")]
    pub is_for_sale: bool, // If true, this serial number is for sale. If false, this serial number is not currently for sale.
    #[serde(rename = "MinBidAmountNanos")]
    pub price: u128, // Minimum bid amount in nanos allowed on this serial number.
    #[serde(rename = "IsBuyNow")]
    pub is_buy_now: bool, // If true, this serial number can be purchased at the price of BuyNowPriceNanos without requiring an accept nft bid transaction from the owner.
    #[serde(rename = "BuyNowPriceNanos")]
    pub buy_now_price: u128, // This is the price at which this serial number can be "bought now". A user can "Buy Now" by submitting a bid that matches the buy now price nanos.
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PostEntryResponse {
  #[serde(rename = "PostHashHex")]
  pub post_hash_hex: String, // Hex of the Post Hash. Used as the unique identifier of this post.
  #[serde(rename = "PosterPublicKeyBase58Check")]
  pub poster_public_key: String,
  #[serde(rename = "Body")]
  pub body: String,
  #[serde(rename = "ImageURLs")]
  pub image_urls: Option<Vec<String>>,
  #[serde(rename = "HasUnlockable")]
  pub has_unlockable: bool,
  #[serde(rename = "PostExtraData")]
  pub extra_data: HashMap<String, String>,
  #[serde(rename = "NumNFTCopies")]
  #[serde(default)]
  pub copies_minted: u64,
  #[serde(rename = "TimestampNanos")]
  pub timestamp: u128,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Associations {
    #[serde(rename = "Associations")]
    pub associations: Vec<Association>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Association {
    #[serde(rename = "AssociationID")]
    pub association_id: String,
    #[serde(rename = "TransactorPublicKeyBase58Check")]
    pub transactor_public_key_base58_check: String,
    #[serde(rename = "TargetUserPublicKeyBase58Check")]
    pub target_user_public_key_base58_check: String,
    #[serde(rename = "AppPublicKeyBase58Check")]
    pub app_public_key_base58_check: String,
    #[serde(rename = "AssociationType")]
    pub association_type: String,
    #[serde(rename = "AssociationValue")]
    pub association_value: String,
    #[serde(rename = "ExtraData")]
    pub extra_data: Option<HashMap<String, String>>,
    #[serde(rename = "BlockHeight")]
    pub block_height: u32,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct UserAssociationQuery {
    #[serde(rename = "TransactorPublicKeyBase58Check")]
    pub transactor_public_key_base58_check: Option<String>,
    #[serde(rename = "TargetUserPublicKeyBase58Check")]
    pub target_user_public_key_base58_check: Option<String>,
    #[serde(rename = "AppPublicKeyBase58Check")]
    pub app_pub_key: Option<String>,
    #[serde(rename = "AssociationType")]
    pub association_type: Option<String>,
    #[serde(rename = "AssociationTypePrefix")]
    pub association_type_prefix: Option<String>,
    #[serde(rename = "AssociationValue")]
    pub association_value: Option<String>,
    #[serde(rename = "AssociationValuePrefix")]
    pub association_value_prefix: Option<String>,
    #[serde(rename = "AssociationValues")]
    pub association_values: Option<Vec<String>>,
    #[serde(rename = "Limit")]
    pub limit: Option<i32>,
    #[serde(rename = "LastSeenAssociationID")]
    pub last_seen_association_id: Option<String>,
    #[serde(rename = "SortDescending")]
    pub sort_descending: Option<bool>,
    #[serde(rename = "IncludeTransactorProfile")]
    pub include_transactor_profile: Option<bool>,
    #[serde(rename = "IncludeTargetUserProfile")]
    pub include_target_profile: Option<bool>,
    #[serde(rename = "IncludeAppProfile")]
    pub include_app_profile: Option<bool>
}

impl UserNftsRequest {
    async fn run(request: &UserNftsRequest, client: &reqwest::Client) -> Result<NFTs, FailureResponse> {
        let uri = "https://node.deso.org/api/v0/get-nfts-for-user";


        let resp = match client.post(uri).json(request).send().await {
            Ok(r) => r,
            Err(e) => return Err(FailureResponse {
                body: format!("Failed to send nft response: {}", e)
            }),
        };

        let text = match resp.text().await {
            Ok(t) => t,
            Err(e) => return Err(FailureResponse {
                body: format!("Failed to get all nfts: {}", e)
            }),
        };

        let nfts: NFTs = match serde_json::from_str(&text) {
            Ok(j) => j,
            Err(e) => return Err(FailureResponse {
                body: format!("Failed to parse all nfts: {}", e)
            }),
        };
        Ok(nfts)
    }
}
impl UserAssociationQuery {
    async fn run(query: &UserAssociationQuery) -> Result<Associations, FailureResponse> {
        let client = reqwest::Client::new();
        let uri = "https://node.deso.org/api/v0/user-associations/query";

        let text = match run(&client, uri, query).await {
            Ok(t) => t,
            Err(e) => return Err(FailureResponse {
                body: format!("Failed to get all associations: {}", e)
            }),
        };
        let associations: Associations = match serde_json::from_str(&text) {
            Ok(j) => j,
            Err(e) => return Err(FailureResponse {
                body: format!("Failed to get all associations: {}", e)
            }),
        };
        Ok(associations)
    }
}

pub async fn run<T: Serialize + ?Sized>(client: &reqwest::Client, uri: &str, json: &T) -> Result<String, FailureResponse> {
    let all_associations = match client.post(uri).json(json).send().await {
        Ok(r) => r,
        Err(e) => return Err(FailureResponse {
            body: format!("Failed to get something: {}", e)
        }),
    };
    let text = match all_associations.text().await {
        Ok(t) => t,
        Err(e) => return Err(FailureResponse {
            body: format!("Failed to get something: {}", e)
        }),
    };

    Ok(text)
}

// Implement Display for the Failure response so that we can then implement Error.
impl std::fmt::Display for FailureResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.body)
    }
}

// Implement Error for the FailureResponse so that we can `?` (try) the Response
// returned by `lambda_runtime::run(func).await` in `fn main`.
impl std::error::Error for FailureResponse {}

type Response = Result<SuccessResponse, FailureResponse>;

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    let func = handler_fn(handler);
    lambda_runtime::run(func).await?;

    Ok(())
}

async fn get_associations() -> Result<Associations, FailureResponse> {
    let client = reqwest::Client::new();

    let uri = "http://spatiumtest-env.eba-wke3mfsm.us-east-1.elasticbeanstalk.com/api/author-associations";

    let resp = match client.get(uri).send().await {
        Ok(r) => r,
        Err(e) => return Err(FailureResponse {
            body: e.to_string()
        }),
    };

    let text = match resp.text().await {
        Ok(t) => t,
        Err(e) => return Err(FailureResponse {
            body: e.to_string()
        }),
    };

    let associations: Associations = match serde_json::from_str(&text.to_string()) {
        Ok(j) => j,
        Err(e) => return Err(FailureResponse {
            body: e.to_string()
        }),
    };

    Ok(associations)

}

async fn check_author_nft(author_key: String, association_id: String, client: reqwest::Client) -> Response {
    let spatium_user_key = String::from("BC1YLg9piUDwrwTZfRipfXNq3hW3RZHW3fJZ7soDNNNnftcqrJvyrbq");
    
    // 1. Get all NFTs for author
    let request = UserNftsRequest {
        user_public_key: author_key
    };

    let nfts: NFTs = UserNftsRequest::run(&request, &client).await?;


    for nft_data in nfts.nfts.values() {
        if nft_data.post.poster_public_key.eq(&spatium_user_key) {
            // Check if nft is author AND not expired
            let extra_data = &nft_data.post.extra_data;
            if extra_data.contains_key("expiration_date") && extra_data.contains_key("nft_type") {
                let expired = match extra_data.get(&String::from("expiration_date")) {
                    Some(d) => is_expired(d.to_string()).await,
                    None => true
                };
                let nft_type = match extra_data.get(&String::from("AUTHOR")) {
                    Some(n) => n.eq(&"Spatium Author"),
                    None => false
                };
                if expired && nft_type {
                    // Remove associatioin
                    let uri = format!("http://spatiumtest-env.eba-wke3mfsm.us-east-1.elasticbeanstalk.com/api/remove-author-association/{}", association_id);
                    match client.post(&uri).send().await {
                        Ok(_) => println!("Removed association successfully {}", association_id),
                        Err(e) => println!("Error! {}", e)
                    }
                }
            }
        }
    }
    Ok(SuccessResponse {
        body: String::from("Success!")
    })   
}

async fn is_expired(expiration_date: String) -> bool {
    let now: i64 = time::OffsetDateTime::now_utc().unix_timestamp();
    let unix_timestamp = expiration_date.parse().expect("Could not parse unix timestamp");
    now > unix_timestamp
}

async fn handler(_event: Value, _ctx: lambda_runtime::Context) -> Response {
    let transactor_public_key_base58_check = Some(String::from("BC1YLg9piUDwrwTZfRipfXNq3hW3RZHW3fJZ7soDNNNnftcqrJvyrbq"));
    let association_type = Some(String::from("Spatium Author"));
    
    // 1. Get all Spatium Author associations
    let associations: Associations = match get_associations().await {
        Ok(a) => a,
        Err(e) => {
            return Err(FailureResponse {
                body: e.to_string(),
            });
        }
    };

    // 2. For each author, check the NFTs they own and find the Spatium Author NFT

    let tasks: Vec<_> = associations.associations
    .iter()
    .map(|association| {
        let author_key = association.target_user_public_key_base58_check.clone();
        let association_id = association.association_id.clone();
        tokio::spawn(async move {
            match check_author_nft(author_key, association_id, reqwest::Client::new()).await {
                Ok(_) => println!("Checked Author!"),
                Err(e) => println!("Error: {}", e.to_string())
            };
        })
    })
    .collect();
    join_all(tasks).await;

    Ok(SuccessResponse {
        body: String::from("Success!"),
    })
}
