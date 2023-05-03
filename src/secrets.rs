use crate::models::Conns;
use aws_config::SdkConfig;
use lambda_runtime::Error;
use serde::Deserialize;
use tokio_postgres::config::SslMode;
use tokio_postgres::Config;

#[derive(Deserialize, Debug)]
pub struct DbConnSecret {
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub dbname: String,
}

impl DbConnSecret {
    pub fn to_db_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}?sslmode=require",
            self.username, self.password, self.host, self.port, self.dbname,
        )
    }
}

pub async fn get_conn_info(conf: &SdkConfig) -> Result<(DbConnSecret, Conns), Error> {
    let client = aws_sdk_secretsmanager::Client::new(conf);

    let db_fut = client.get_secret_value().secret_id("pknockerdb").send();
    let conns_fut = client.get_secret_value().secret_id("pknockerConns").send();

    let db =
        serde_json::from_str::<DbConnSecret>(db_fut.await?.secret_string().unwrap_or_default())?;

    let conns =
        serde_json::from_str::<Conns>(conns_fut.await?.secret_string().unwrap_or_default())?;

    Ok((db, conns))
}

// pub async fn get_inst() -> Result<(), Error> {
//     Ok(())
// let client = aws_sdk_ec2::Client::new(&shared_config);
// let instances = client.describe_instances().send().await?;
//
// for res in instances.reservations().unwrap_or_default() {
//     for instance in res.instances().unwrap_or_default() {
//         let name = instance.key_name().unwrap_or_default();
//         let ip = instance.public_ip_address().unwrap_or_default();
//         println!("{name} :: {ip}");
//     }
// }
//
// Ok(())
// }
