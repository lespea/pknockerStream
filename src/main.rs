#![allow(unused_imports)]

use std::io::Cursor;
use std::ops::Deref;
use std::str::FromStr;
use std::time::Duration;

use chrono::Utc;
use diesel::debug_query;
use diesel::dsl::{exists, not};
use diesel::prelude::*;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use ipnetwork::IpNetwork;
use lambda_runtime::Error;
use once_cell::sync::Lazy;
use tokio_postgres_rustls::MakeRustlsConnect;
use tracing::log::{debug, info};

use crate::models::Block;
use crate::models::Conns;
use crate::models::InetProto::{Tcp, Udp};
use crate::schema::added;
use crate::schema::denies::dsl::denies;

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
    if PRINT_WANTED {
        let out = serde_json::to_string(WANTED_CONNS.deref())?;
        println!("{out}");
        return Ok(());
    }

    init();
    let conf = aws_config::from_env().region("us-east-1").load().await;

    let (db_conn_info, wanted) = secrets::get_conn_info(&conf).await?;

    if true {
        println!("{wanted:?}");
        return Ok(());
    }

    let mgr = AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_setup(
        db_conn_info.to_db_url(),
        establish_connection,
    );

    let pool = Pool::builder(mgr).max_size(2).build()?;

    use self::schema::blocks;
    use self::schema::denies;
    use crate::models::*;

    let mut conn = pool.get().await?;

    if true {
        if false {
            let query = blocks::table
                .left_outer_join(denies.on(blocks::src_ip.eq(denies::ip)))
                .filter(not(blocks::port.eq(22).and(blocks::proto.eq(Tcp))))
                .filter(denies::ip.is_null())
                .order(blocks::event_ts.asc())
                .select((
                    blocks::id,
                    blocks::src_ip,
                    blocks::dst_ip,
                    blocks::proto,
                    blocks::port,
                    blocks::event_ts,
                    blocks::insert_ts,
                ));

            info!("{:?}", debug_query::<diesel::pg::Pg, _>(&query));

            for block in query.load::<Block>(&mut conn).await? {
                println!("Block: {block:?}");
            }
        } else {
            for to_check in view_to_check::table.load::<ViewToCheck>(&mut conn).await? {
                println!("{to_check:?}");
                let conns = serde_json::from_str::<Conns>(&to_check.conns).unwrap();
                for v in conns.0.iter() {
                    println!("CONN={v:?}");
                }
                println!("== {}", conns == *WANTED_CONNS);
            }
        }
    } else {
        let localip = IpNetwork::from_str("127.0.0.1").unwrap();
        let otherip1 = IpNetwork::from_str("10.123.21.1").unwrap();
        let otherip2 = IpNetwork::from_str("172.16.5.4").unwrap();

        let dstip = IpNetwork::from_str("10.99.88.44").unwrap();

        let mut new_block = NewBlock {
            src_ip: localip,
            dst_ip: dstip,
            event_ts: Utc::now(),
            proto: Tcp,
            port: 55,
        };

        for ip in [localip, otherip1, otherip2] {
            new_block.src_ip = ip;

            for proto in [Tcp, Udp] {
                new_block.proto = proto;
                for port in [22, 443, 8080] {
                    new_block.port = port;

                    for block in diesel::insert_into(blocks::table)
                        .values(&new_block)
                        .get_results::<Block>(&mut conn)
                        .await
                        .expect("Bad insert")
                    {
                        println!("Inserted {block:?}");
                    }
                }
            }
        }

        diesel::insert_into(denies::table)
            .values(denies::ip.eq(localip))
            .execute(&mut conn)
            .await
            .expect("Bad insert");

        diesel::insert_into(added::table)
            .values((added::src_ip.eq(otherip2), added::dst_ip.eq(dstip)))
            .execute(&mut conn)
            .await
            .expect("Bad insert");
    }

    Ok(())
}
