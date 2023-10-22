use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use lambda_runtime::{LambdaEvent};
use std::collections::HashMap;
use std::env;
use lambda_http::{service_fn, Response, Body, Error, Request};
use lambda_http::http::StatusCode;
use serde_json::json;
use reqwest;
use uuid::Uuid;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client as DbClient;
use aws_sdk_dynamodb::{config::Region, meta::PKG_VERSION};
use aws_config::{meta::region::RegionProviderChain, SdkConfig};


/**
 * All Structs Needed for messing with Plaid API and AWS
 */
#[derive(Serialize, Debug)]
pub struct User {
    pub client_user_id: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MyRequest {
    pub public_token: String,
    pub financial_inst: String
}

#[derive(Serialize, Debug)]
 pub struct CreateAccessTokenRequest {
    pub client_id: String,
    pub secret: String,
    pub public_token: String
 }

 impl CreateAccessTokenRequest {
    pub async fn create(public_token: String) -> CreateAccessTokenRequest {
        CreateAccessTokenRequest {
            client_id: get_plaid_client_key().await.unwrap(),
            secret: get_plaid_api_key().await.unwrap(),
            public_token: public_token
        }
    }
 }

 #[derive(Deserialize)]
 pub struct CreateAccessTokenResponse {
    pub access_token: String
 }

 #[derive(Debug)]
pub struct Opt {
    /// The AWS Region.
    pub region: Option<String>,
    /// Whether to display additional information.
    pub verbose: bool,
}

 /**
  * All Helper Functions
  */
async fn get_plaid_api_key() -> Option<String> {
    env::var("PLAID_API_KEY").ok()
}

async fn get_plaid_client_key() -> Option<String> {
    env::var("PLAID_CLIENT_ID").ok()
}

async fn get_db_table_name() -> Option<String> {
    env::var("TABLE_NAME").ok()
}

async fn generate_uuid() -> String {
    Uuid::new_v4().to_string()
}

pub fn make_region_provider(region: Option<String>) -> RegionProviderChain {
    RegionProviderChain::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-east-1"))
}

async fn make_config(opt: Opt) -> Result<SdkConfig, Error> {
    let region_provider = make_region_provider(opt.region);

    Ok(aws_config::from_env().region(region_provider).load().await)
}

async fn add_to_db(response: CreateAccessTokenResponse, financial_inst: String) -> Result<(), Error> {
    let opt = Opt {
        region: Some("us-east-1".to_string()),
        verbose: true,
    };
    let config = match make_config(opt).await {
        Ok(c) => c,
        Err(e) => {
            return Err(Error::from(format!("Error making config: {}", e.to_string())));
        },
    };
    let db_client = DbClient::new(&config);
    let table_name = match get_db_table_name().await {
        Some(t) => t,
        None => {
            return Err(Error::from(format!(
                "DB Name Not Set"
            )));
        }
    };

    let uuid = AttributeValue::S(generate_uuid().await);
    let f_inst = AttributeValue::S(financial_inst);
    let access_token = AttributeValue::S(response.access_token);

    let request = db_client
        .put_item()
        .table_name(table_name)
        .item("uuid", uuid)
        .item("financial_institution", f_inst)
        .item("access_token", access_token);

    println!("Executing request [{request:?}] to add item...");

    request.send().await?;
    Ok(())
}

  /**
   * Main Flow calls the handler function
   */
#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(handler);
    lambda_http::run(func).await?;

    Ok(())
}

async fn handler(req: Request) -> Result<Response<String>, Error> {
    // Create Access token request object
    let body = req.body();
    let user_request: MyRequest = serde_json::from_slice(&body)?;

    let request = CreateAccessTokenRequest::create(user_request.public_token).await;
    let uri = "https://development.plaid.com/item/public_token/exchange";
    let client = reqwest::Client::new();

    // Make a POST request to the URI
    let response = client
    .post(uri)
    .json(&request)
    .send()
    .await
    .map_err(|err| {
        println!("Error making POST request: {:?}", err);
        Error::from(err)
    })?;

    // Check if the request was successful (HTTP status code 200)
    if response.status().is_success() {
        // Add access token to DB

        // Deserialize the JSON response into the CreateLinkTokenResponse struct
        let response_body = response.text().await.map_err(|err| {
            println!("Error reading response body: {:?}", err);
            Error::from(err)
        })?;

        let access_token_response: CreateAccessTokenResponse = serde_json::from_str(&response_body).map_err(|err| {
            println!("Error deserializing response body: {:?}", err);
            Error::from(err)
        })?;

        add_to_db(access_token_response, user_request.financial_inst).await?;

        // Return a successful response with the link token
        Ok(Response::builder()
            .status(200)
            .header("Access-Control-Allow-Origin", "*")
            .body(String::from("Ok"))?)
    } else {
        // Handle error cases here (e.g., return an error response or log the error)
        Err(Error::from(format!(
            "Request failed with status code: {}",
            response.status()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_handler() {
        dotenv::from_filename("../../.env").ok();
        // Provide mock values for public_token and financial_inst
        let public_token = "link-sandbox-813f38c9-9570-4111-9ad5-f1e0c7f2defe";
        let financial_inst = "Wells Fargo";
        let user_request = MyRequest {
            public_token: public_token.to_string(),
            financial_inst: financial_inst.to_string(),
        };
        
        // Serialize the mock user_request into JSON
        let user_request_json = serde_json::to_string(&user_request).unwrap();
        
        // Create a mock request with the serialized user_request JSON
        let req = Request::default()
            .from(Body::Text(user_request_json));

        // Call handler function with mock request
        let response = handler(req).await;

        // Assert response is OK
        assert!(response.is_ok());

        let response = response.unwrap();

        // Assert response code is 200
        assert_eq!(response.status(), StatusCode::OK);

        println!("{:?}", response);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_handler() {
        dotenv::from_filename("../../.env").ok();
        let req = Request::default();
        // pub struct MyRequest {
        //     pub public_token: String,
        //     pub financial_inst: String
        // }
        let response = handler(req).await;
        assert!(response.is_ok());
        let response = response.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        println!("{:?}", response);
    }
}
