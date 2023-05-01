#![allow(unused_imports)]

mod models;
mod schema;

// use aws_lambda_events::event::s3::S3Event;
// use lambda_runtime::{run, service_fn, Error, LambdaEvent};

use crate::models::Block;

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::Text;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use dotenvy::dotenv;
use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use ipnetwork::IpNetwork;
use lambda_runtime::Error;
use once_cell::sync::Lazy;
use postgres_native_tls::MakeTlsConnector;
use std::env;
use std::io::Cursor;
use std::str::FromStr;
use std::time::Duration;
use tokio_postgres::Config;
use tokio_postgres_rustls::MakeRustlsConnect;
use tracing::instrument::WithSubscriber;
use tracing::log::{debug, info};

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
// async fn function_handler(event: LambdaEvent<S3Event>) -> Result<(), Error> {
// Extract some useful information from the request

// Ok(())
// }

static ROOT_CERT: Lazy<MakeRustlsConnect> = Lazy::new(|| {
    let mut root = rustls::RootCertStore::empty();
    let b = include_bytes!("us-east-1-bundle.pem");
    root.add_parsable_certificates(&rustls_pemfile::certs(&mut Cursor::new(b)).expect("bad cert"));

    MakeRustlsConnect::new(
        rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root)
            .with_no_client_auth(),
    )
});

fn init() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    dotenv().unwrap();
}

fn establish_connection(url: &str) -> BoxFuture<ConnectionResult<AsyncPgConnection>> {
    let fut = async {
        let (client, conn) = tokio_postgres::connect(url, ROOT_CERT.clone())
            .await
            .map_err(|e| ConnectionError::BadConnection(e.to_string()))
            .expect("Bad conn");

        tokio::spawn(async move {
            if let Err(e) = conn.await {
                eprintln!("Database connection: {e}");
            }
        });

        AsyncPgConnection::try_from(client).await
    };

    fut.boxed()
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    init();

    let url = env::var("DATABASE_URL").expect("No db url env");

    let mgr = AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_setup(
        url,
        establish_connection,
    );

    let pool = Pool::builder(mgr).max_size(2).build()?;

    if false {
        let mut conn = pool.get().await?;

        use self::models::*;
        use self::schema::blocks::dsl::*;

        for block in blocks.load::<Block>(&mut conn).await? {
            println!("{block:?}")
        }
    } else {
        let mut conn = pool.get().await?;

        use self::models::*;
        use self::schema::blocks;

        let new_block = NewBlock {
            src_ip: IpNetwork::from_str("127.0.0.1")?,
            dst_ip: IpNetwork::from_str("10.99.88.44")?,
            event_ts: Utc::now(),
            proto: InetProto::Tcp,
        };

        for block in diesel::insert_into(blocks::table)
            .values(&new_block)
            .get_results::<Block>(&mut conn)
            .await
            .expect("Bad insert")
        {
            println!("Inserted {block:?}");
        }
    }

    Ok(())
}
