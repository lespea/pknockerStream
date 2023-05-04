use std::io::Cursor;
use std::str::FromStr;

use chrono::Utc;
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
use tracing::log::{error, info, warn};

use crate::models::*;
use crate::schema::*;
use crate::secrets::DbConnSecret;

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

pub async fn get_pool(db_conn_info: DbConnSecret) -> Result<Pool<AsyncPgConnection>, Error> {
    let mgr = AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_setup(
        db_conn_info.to_db_url(),
        establish_connection,
    );

    Ok(Pool::builder(mgr).max_size(2).build()?)
}

pub async fn clean(pool: &Pool<AsyncPgConnection>) -> Result<(), Error> {
    info!("Cleaning db");
    let mut conn = pool.get().await?;
    diesel::sql_query("CALL clean_db();")
        .execute(&mut conn)
        .await?;
    Ok(())
}

pub async fn run_checks(pool: &Pool<AsyncPgConnection>) -> Result<(), Error> {
    info!("Run checks");
    let mut conn = pool.get().await?;

    for to_check in view_to_check::table.load::<ViewToCheck>(&mut conn).await? {
        let src = to_check.src_ip;

        let mut conns = serde_json::from_str::<Conns>(&to_check.conns)?;
        conns.0.sort();
        if conns == *crate::WANTED_CONNS {
            match crate::ec2::get_ip_map().await.get(&to_check.src_ip) {
                Some(info) => {
                    if let Err(err) = crate::ec2::add_allow(to_check.src_ip, info).await {
                        error!("Couldn't allow {src}: {err}")
                    }
                }
                None => {
                    // if let Err(err) = add_deny(to_check, pool).await {
                    warn!("Unknown src ip {}", to_check.src_ip);
                    // };
                }
            };
        } else if crate::WANTED_CONNS.should_block(conns) {
            if let Err(err) = add_deny(to_check, pool).await {
                error!("Couldn't insert {src} into the db: {err}")
            }
        };
    }
    Ok(())
}

pub async fn add_deny(check: ViewToCheck, pool: &Pool<AsyncPgConnection>) -> Result<(), Error> {
    let mut conn = pool.get().await?;

    diesel::insert_into(denies::table)
        .values(denies::ip.eq(check.src_ip))
        .execute(&mut conn)
        .await?;

    Ok(())
}

pub async fn insert_test_data(pool: &Pool<AsyncPgConnection>) -> Result<(), Error> {
    use crate::models::InetProto::*;

    let mut conn = pool.get().await?;

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

    diesel::insert_into(added::table)
        .values((added::src_ip.eq(otherip2), added::dst_ip.eq(dstip)))
        .execute(&mut conn)
        .await
        .expect("Bad insert");

    diesel::insert_into(denies::table)
        .values(denies::ip.eq(localip))
        .execute(&mut conn)
        .await
        .expect("Bad insert");

    Ok(())
}
