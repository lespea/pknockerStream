#![allow(unused_imports)]

use std::io::Cursor;
use std::ops::Deref;
use std::str::FromStr;
use std::time::Duration;

use aws_config::SdkConfig;
use lambda_runtime::Error;
use once_cell::sync::{Lazy, OnceCell};
use tracing::log::{debug, info};

use crate::models::Block;
use crate::models::Conns;
use crate::models::InetProto::{Tcp, Udp};
use crate::schema::added;

mod db;
mod models;
mod schema;
mod secrets;

// use aws_lambda_events::event::s3::S3Event;
// use lambda_runtime::{run, service_fn, Error, LambdaEvent};

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
// async fn function_handler(event: LambdaEvent<S3Event>) -> Result<(), Error> {
// Extract some useful information from the request

// Ok(())
// }

const PRINT_WANTED: bool = false;

static WANTED_CONNS: Lazy<Conns> = Lazy::new(|| {
    Conns(vec![
        (Tcp, 7614),
        (Udp, 1234),
        (Tcp, 9971),
        (Udp, 1234),
        (Udp, 23657),
        (Tcp, 9911),
    ])
});

static AWS_CONF: OnceCell<SdkConfig> = OnceCell::new();

#[tokio::main]
async fn main() -> Result<(), Error> {
    if PRINT_WANTED {
        let out = serde_json::to_string(WANTED_CONNS.deref())?;
        println!("{out}");
        return Ok(());
    }

    init().await;

    let (db_conn_info, _) = secrets::get_conn_info().await?;
    let pool = db::get_pool(db_conn_info).await?;

    if false {
        db::insert_test_data(&pool).await?;
    }
    if true {
        db::run_checks(&pool).await?;
    }

    Ok(())
}

async fn init() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let conf = aws_config::from_env().region("us-east-1").load().await;
    AWS_CONF.set(conf).unwrap();
}
