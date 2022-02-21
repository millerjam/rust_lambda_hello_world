# Creating AWS Lambdas in Rust

## Getting started with AWS Lambda Rust Runtime

Quick tutorial to create a sample "hello world" lambda using 
[AWS Lambda Rust Runtime](https://github.com/awslabs/aws-lambda-rust-runtime)

## Pre-requisite
1. Install rust/cargo/etc
1. Code editor
1. AWS Account

## Create a new rust binary

```
cargo new lambda_test
```

## Add the code

To start with something easy lets copy the code from the [AWS Lambda Rust Runtime](https://github.com/awslabs/aws-lambda-rust-runtime/tree/v0.5.0) README example. (Note: make sure you've selected the tag for version, v0.5 at the time I'm writing this)

`main.rs`
```
use lambda_runtime::{service_fn, LambdaEvent, Error};
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(event: LambdaEvent<Value>) -> Result<Value, Error> {
    let (event, _context) = event.into_parts();
    let first_name = event["firstName"].as_str().unwrap_or("world");

    Ok(json!({ "message": format!("Hello, {}!", first_name) }))
}
```

## Fix the copy pasta errors

Currently the lambda will not build, your IDE may show some crate not found errors.

Lets update the `Cargo.toml` with the required libraries.

I use [Cargo Edit](https://github.com/killercup/cargo-edit) which extends Cargo with commands to manage dependencies without having to manually edit your `Cargo.toml`

```
cargo add lambda_runtime

cargo add serde_json

cargo add tokio
```

Finally, the project should be ready to build.

## How to build

### Check that normal local builds are working, and fix any dependency issues if needed.

```
cargo build
```


### Build for AWS Lambda install x86 target

* First install Zig

```
brew install zig
```

* Install cargo zigbuild
```
cargo install cargo-zigbuild
```

* Add the target for x86
```
rustup target add x86_64-unknown-linux-gnu
```

* Finally, we build the release

```
cargo zigbuild --release --target x86_64-unknown-linux-gnu
```

### Create a zip archive

```
cp ./target/x86_64-unknown-linux-gnu/release/lambda_test ./bootstrap && zip lambda.zip bootstrap && rm bootstrap
```

### Create new function
`TODO: how to create the role, then in IAM can copy ARN`

```
aws lambda create-function --function-name rustTest \
  --handler doesnt.matter \
  --zip-file fileb://./lambda.zip \
  --runtime provided.al2 \
  --role arn:aws:iam::{PUT_AWS_ACCT}:role/lambda_basic_execution \
  --environment Variables={RUST_BACKTRACE=1} \
  --tracing-config Mode=Active
  ```

### Test invoke

```
aws lambda invoke \
  --cli-binary-format raw-in-base64-out \
  --function-name rustTest \
  --payload '{"firstName": "James"}' \
  output.json
```

### Check output
```
> cat output.json
{"message":"Hello, James!"}%
```

### Check logs
`TODO: how to check cloudwatch logs`

```
START RequestId: 2b0a5ad0
Version: $LATEST
END RequestId: 2b0a5ad0
REPORT RequestId: 2b0a5ad0	Duration: 0.85 ms	Billed Duration: 1 ms	Memory Size: 128 MB	Max Memory Used: 15 MB	
XRAY TraceId: 1-6213be54	SegmentId: 282cd	Sampled: true	

```