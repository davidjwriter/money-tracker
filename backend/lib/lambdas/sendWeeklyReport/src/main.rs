use lambda_runtime::{LambdaEvent};
use lambda_http::{service_fn, Response, Body, Error, Request};
use lambda_http::http::StatusCode;
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