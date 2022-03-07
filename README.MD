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
```rust
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
`TODO: how to check cloudwatch logs from cli?`

```
START RequestId: 2b0a5ad0
Version: $LATEST
END RequestId: 2b0a5ad0
REPORT RequestId: 2b0a5ad0	Duration: 0.85 ms	Billed Duration: 1 ms	Memory Size: 128 MB	Max Memory Used: 15 MB	
XRAY TraceId: 1-6213be54	SegmentId: 282cd	Sampled: true	

```
## Lets add some logging 

Before we do too much more lets add some logging. We are take advantage of the tracing library which is already integrated into many of the lambda dependencies.

`TODO: add more details about tracing`

### Init the logger
Setting up the logger requires initializing the tracing subscriber library, the [AWS Lambda Rust Runtime Examples](https://github.com/awslabs/aws-lambda-rust-runtime/tree/v0.5.0/lambda-runtime/examples) have a examples with some details about how to get started.

Here is the code that we will have to add:
```rust
fn init_lambda_tracing() {
    tracing_subscriber::fmt()        
        .with_max_level(tracing::Level::INFO)
        // this needs to be set to false, otherwise ANSI color codes will
        // show up in a confusing manner in CloudWatch logs.
        .with_ansi(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();
}
```

This function will setup the default `tracing_subscriber` which prints logs to stdout. 
The additional methods called on the builder, set the default logging level to "info"
and disable some of the default to make things nicer in the Lambda environment. I added
an additional line to the `main()` to call this function before the lambda runtime setup.

### Fix the compile issues

Even though we did not add any additional `import ...` lines, we still need to add our new tracing library dependencies to Cargo, so our code will build.
The reason we did not add `import..` statements is that we fully qualified the crate directly in the code with the prefix `tracing_subscriber::`.

Lets go ahead and add the 2 dependencies: [tracing](https://crates.io/crates/tracing), and [tracing-subscriber](https://crates.io/crates/tracing-subscriber)

```
cargo add tracing

cargo add tracing-subscriber

```
### Lets test it!

Ok now that we have logging setup, let's add some logs and try it.

Add the additional log to the `func` method

```rust
info!("Going to say hello to {}!", first_name);
```

We will also need to import
```rust
use tracing::info;
```

Now we can update the function and test, 

### Compile & Update the function
```
TODO COMPILE STUFF

aws lambda update-function-code --function-name rustTest \
  --zip-file fileb://./lambda.zip
  ```

### Test invoke

```
aws lambda invoke \
  --cli-binary-format raw-in-base64-out \
  --function-name rustTest \
  --payload '{"firstName": "James"}' \
  output.json
```
### Now check the cloudwatch logs again

```
START RequestId: 93bcbe8c-4b2e-4af3-b00d-3b4b27083ebe Version: $LATEST
INFO lambda_test: going to say hello to james
END RequestId: 93bcbe8c-4b2e-4af3-b00d-3b4b27083ebe
REPORT RequestId: 93bcbe8c-4b2e-4af3-b00d-3b4b27083ebe	Duration: 1.16 ms	Billed Duration: 33 ms	Memory Size: 128 MB	Max Memory Used: 17 MB	Init Duration: 31.20 ms	
XRAY TraceId: 1-622553a3-5644934610a4952626f10403	SegmentId: 3b7b79993cba7894	Sampled: true	
```

Success! We now have our log, `INFO lambda_test: going to say hello to james`


### Reconfigure the logger

Now that we have logging working, lets reconfigure it so that it reads the log level
from an environment variable, that will give us the ability to turn logging levels up and down while we are developing and debugging.

We can update the `init_lambda_tracing` like this:
```rust
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
```

We replaced the hardcoded `set_max_level...` with `.with_env_filter(EnvFilter::from_default_env())` this will read the logging level from the 
environment variable `RUST_LOG`. Now we can change the log level dynamically without having the recompile and update the running code.



### Update function configuration with DEBUG log level

Now that code is update, we need to rebuild, update the function, and then also the config
to add the new "RUST_LOG" environment variable.
```
TODO COMPILE STUFF, UPDATE FUNCTION, then Configuration

aws lambda update-function-configuration \
    --function-name  rustTest \
    --environment Variables="{RUST_BACKTRACE=1,RUST_LOG=info}"
```
### Test invoke again

```
aws lambda invoke \
  --cli-binary-format raw-in-base64-out \
  --function-name rustTest \
  --payload '{"firstName": "James"}' \
  output.json
```
### Check the cloudwatch logs again
Logs are much more verbose, we can see lots of new logs from the included dependencies.

```
START RequestId: 9ebad5dc-3cee-4302-a0f8-0e592e432f41 Version: $LATEST
DEBUG hyper::client::connect::http: connecting to 127.0.0.1:9001
DEBUG hyper::client::connect::http: connected to 127.0.0.1:9001
DEBUG hyper::proto::h1::io: flushed 109 bytes
DEBUG hyper::proto::h1::io: parsed 7 headers
DEBUG hyper::proto::h1::conn: incoming body is content-length (21 bytes)
DEBUG hyper::proto::h1::conn: incoming body completed
DEBUG hyper::client::pool: pooling idle connection for ("http", 127.0.0.1:9001)
INFO lambda_test: going to say hello to james
DEBUG hyper::client::pool: reuse idle connection for ("http", 127.0.0.1:9001)
DEBUG hyper::proto::h1::io: flushed 198 bytes
DEBUG hyper::proto::h1::io: parsed 3 headers
DEBUG hyper::proto::h1::conn: incoming body is content-length (16 bytes)
DEBUG hyper::client::connect::http: connecting to 127.0.0.1:9001
DEBUG hyper::proto::h1::conn: incoming body completed
DEBUG hyper::client::pool: reuse idle connection for ("http", 127.0.0.1:9001)
DEBUG hyper::proto::h1::io: flushed 109 bytes
END RequestId: 9ebad5dc-3cee-4302-a0f8-0e592e432f41
REPORT RequestId: 9ebad5dc-3cee-4302-a0f8-0e592e432f41	Duration: 1.46 ms	Billed Duration: 38 ms	Memory Size: 128 MB	Max Memory Used: 17 MB	Init Duration: 36.11 ms	
XRAY TraceId: 1-622551be-6d58480b496655162937c59c	SegmentId: 03a6c5576dc28250	Sampled: true	
DEBUG hyper::client::connect::http: connected to 127.0.0.1:9001
DEBUG hyper::client::pool: pooling idle connection for ("http", 127.0.0.1:9001)
```

