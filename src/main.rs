use std::ops::Deref;

use aws_lambda_events::event::s3::S3Event;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use once_cell::sync::Lazy;
use tracing::log::error;

use crate::models::Conns;
use crate::models::InetProto::{Tcp, Udp};

mod aws;
mod db;
mod ec2;
mod models;
mod parq;
mod s3;
mod schema;
mod secrets;

async fn function_handler(event: LambdaEvent<S3Event>) -> Result<(), Error> {
    let (db_conn_info, _) = secrets::get_conn_info().await?;
    let pool = db::get_pool(db_conn_info).await?;

    if let Err(err) = db::clean(&pool).await {
        error!("Error cleaning: {err}")
    };
    s3::get_and_parse(event.payload, &pool).await;
    db::run_checks(&pool).await
}

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

#[tokio::main]
async fn main() -> Result<(), Error> {
    init().await;

    if PRINT_WANTED {
        print_wanted().await
    } else if false {
        let (db_conn_info, _) = secrets::get_conn_info().await?;
        let pool = db::get_pool(db_conn_info).await?;

        db::insert_test_data(&pool).await?;
        db::run_checks(&pool).await?;

        Ok(())
    } else {
        run(service_fn(function_handler)).await
    }
}

async fn print_wanted() -> Result<(), Error> {
    let out = serde_json::to_string(WANTED_CONNS.deref())?;
    println!("{out}");
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
}
