use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use lambda_runtime::{LambdaEvent};
use std::collections::HashMap;
use std::env;
use lambda_http::{service_fn, Response, Body, Error, Request};
use lambda_http::http::StatusCode;
use serde_json::json;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client as DbClient;
use aws_sdk_dynamodb::{config::Region, meta::PKG_VERSION};
use aws_config::{meta::region::RegionProviderChain, SdkConfig};
  
    #[derive(Debug)]
    pub struct Account {
        pub access_token: String,
        pub financial_inst: String
    }

    pub struct Accounts {
        pub accounts: Vec<Account>
    }

    impl Accounts {
        fn from(items: Vec<HashMap<String, AttributeValue>>) -> Option<Accounts> {
            let mut accounts: Vec<Account> = Vec::new();
            for item in items {
                let access_token = match item.get("access_token") {
                    Some(attribute) => {
                        if let Ok(value) = attribute.as_s() {
                            value.to_string()
                        } else {
                            return None;
                        }
                    }
                    None => return None, // Return early if access_token is missing
                };
        
                let financial_inst = match item.get("financial_institution") {
                    Some(attribute) => {
                        if let Ok(value) = attribute.as_s() {
                            value.to_string()
                        } else {
                            return None;
                        }
                    },
                    None => return None,
                };
                let account = Account {
                    access_token,
                    financial_inst,
                };

                accounts.push(account);
            }
            Some(Accounts {
                accounts,
            })
        }
    }

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

    pub fn make_region_provider(region: Option<String>) -> RegionProviderChain {
    RegionProviderChain::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-east-1"))
    }

    async fn make_config(opt: Opt) -> Result<SdkConfig, Error> {
        let region_provider = make_region_provider(opt.region);

        Ok(aws_config::from_env().region(region_provider).load().await)
    }

    async fn retrieve_accounts() -> Result<Accounts, Error> {
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
        let items: Vec<HashMap<String, AttributeValue>> = db_client
            .scan()
            .table_name(table_name)
            .send().await.expect("Error fetching DB results")
            .items().unwrap().to_vec();

        let accounts: Option<Accounts> = Accounts::from(items);
        if let Some(a) = accounts {
            return Ok(a);
        } else {
            return Err(Error::from(String::from("Error converting items")));
        }
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

    async fn handler(_req: Request) -> Result<Response<Body>, Error> {
        println!("Checking");
        Ok(Response::new(Body::Text("Ok".to_string())))
    }

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_get_items() {
        dotenv::from_filename("../../.env").ok();
        println!("Fetching Items!");
        let accounts = retrieve_accounts().await.expect("Error fetching accounts!");
        for account in accounts.accounts {
            println!("Account: {:?}", account);
        }
    }
}