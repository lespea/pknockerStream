use lambda_runtime::Error;
use serde::Deserialize;
use tracing::log::info;

use crate::models::Conns;

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
        let url = format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.dbname,
        );
        // info!("db url: {url}");
        url
    }
}

pub async fn get_conn_info() -> Result<(DbConnSecret, Conns), Error> {
    let client = aws_sdk_secretsmanager::Client::new(crate::aws::get_conf().await);

    info!("Getting info from secrets");
    let db_fut = client.get_secret_value().secret_id("pknockerdb").send();
    let conns_fut = client.get_secret_value().secret_id("pknockerConns").send();

    info!("Parsing db conn");
    let db =
        serde_json::from_str::<DbConnSecret>(db_fut.await?.secret_string().unwrap_or_default())?;

    info!("Parsing conn list");
    let conns =
        serde_json::from_str::<Conns>(conns_fut.await?.secret_string().unwrap_or_default())?;

    info!("Got info :: {conns:?}");

    Ok((db, conns))
}
