use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::{json, Value};
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Error> {
    init_lambda_tracing();
    let func = service_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

fn init_lambda_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        // this needs to be set to false, otherwise ANSI color codes will
        // show up in a confusing manner in CloudWatch logs.
        .with_ansi(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();
}

async fn func(event: LambdaEvent<Value>) -> Result<Value, Error> {
    let (event, _context) = event.into_parts();
    let first_name = event["firstName"].as_str().unwrap_or("world");
    info!("going to say hello to {}", first_name);
    Ok(json!({ "message": format!("Hello, {}!", first_name) }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lambda_runtime::Context;

    #[tokio::test]
    async fn test_func() {
        let context = Context::default();
        let payload = json!({"firstName": "James"});
        let event = LambdaEvent { payload, context };
        let result = func(event).await.unwrap();
        assert_eq!(result["message"], "Hello, James!");
    }

    #[tokio::test]
    async fn test_func_bad_request() {
        let context = Context::default();
        let payload = json!({"something": "wrong"});
        let event = LambdaEvent { payload, context };
        let result = func(event).await.unwrap();
        assert_eq!(result["message"], "Hello, world!");
    }
}
