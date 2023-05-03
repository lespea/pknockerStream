use aws_config::SdkConfig;
use tokio::sync::OnceCell;

static CONF: OnceCell<SdkConfig> = OnceCell::const_new();

pub async fn get_conf<'a>() -> &'a SdkConfig {
    CONF.get_or_init(|| async { aws_config::from_env().region("us-east-1").load().await })
        .await
}
