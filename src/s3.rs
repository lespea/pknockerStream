use aws_lambda_events::s3::S3Event;
use aws_sdk_s3::Client;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::AsyncPgConnection;
use tracing::log::{error, info};
use urlencoding::decode;

pub async fn get_and_parse(event: S3Event, pool: &Pool<AsyncPgConnection>) {
    let client = Client::new(crate::aws::get_conf().await);

    for rec in event.records {
        let bucket = rec.s3.bucket.name;
        let key = rec
            .s3
            .object
            .key
            .map(|k| decode(&k).unwrap_or_default().to_string());

        info!("Attempting to connect to {bucket:?}/{key:?}");

        match client
            .get_object()
            .set_bucket(bucket)
            .set_key(key)
            .send()
            .await
        {
            Err(err) => error!("Couldn't get bucket: {err:?}"),

            Ok(resp) => match resp.body.collect().await {
                Err(err) => error!("Couldn't get bucket obj: {err:?}"),

                Ok(body) => {
                    if let Err(err) = crate::parq::add_records(body.to_vec(), pool, true).await {
                        error!("Couldn't add block records: {err:?}")
                    }
                }
            },
        }
    }
}
