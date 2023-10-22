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

/**
 * All Structs Needed for messing with Plaid API
 */
#[derive(Serialize, Debug)]
pub struct User {
    pub client_user_id: String
}

#[derive(Serialize, Debug)]
 pub struct CreateLinkTokenRequest {
    pub client_id: String,
    pub secret: String,
    pub client_name: String,
    pub language: String,
    pub country_codes: Vec<String>,
    pub user: User,
    pub products: Vec<String>,
 }

 impl CreateLinkTokenRequest {
    pub async fn create() -> CreateLinkTokenRequest {
        println!("Building Request");
        let user = User {
            client_user_id: generate_uuid().await
        };
        CreateLinkTokenRequest {
            client_id: get_plaid_client_key().await.expect("Failed to get Client ID"),
            secret: get_plaid_api_key().await.expect("Failed to get API Secret"),
            client_name: String::from("Weekly Budget Report"),
            language: String::from("en"),
            country_codes: vec!["US".to_string()],
            user: user,
            products: vec!["transactions".to_string()]
        }
    }
 }

 #[derive(Serialize, Deserialize)]
 pub struct CreateLinkTokenResponse {
    pub link_token: String
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

async fn generate_uuid() -> String {
    Uuid::new_v4().to_string()
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
    let request = CreateLinkTokenRequest::create().await;
    let uri = "https://development.plaid.com/link/token/create";
    let client = reqwest::Client::new();

    println!("Making Request");
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
        // Deserialize the JSON response into the CreateLinkTokenResponse struct
        let response_body = response.text().await.map_err(|err| {
            println!("Error reading response body: {:?}", err);
            Error::from(err)
        })?;

        let link_token_response: CreateLinkTokenResponse = serde_json::from_str(&response_body).map_err(|err| {
            println!("Error deserializing response body: {:?}", err);
            Error::from(err)
        })?;

        // Return a successful response with the link token
        Ok(Response::builder()
            .status(200)
            .header("Access-Control-Allow-Origin", "*")
            .body(serde_json::to_string(&link_token_response).expect("Problem converting response to json"))?)
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
        let req = Request::default();
        let response = handler(req).await;
        assert!(response.is_ok());
        let response = response.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        println!("{:?}", response);
    }
}

