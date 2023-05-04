use crate::aws::get_conf;
use aws_sdk_ec2::types::GroupIdentifier;
use aws_sdk_ec2::Client;
use ipnetwork::IpNetwork;
use lambda_runtime::Error;
use std::collections::HashMap;
use std::str::FromStr;
use tokio::sync::OnceCell;
use tracing::log::{error, info};

pub struct InstanceInfo {
    name: String,
    idents: Vec<String>,
}

static IP_MAP: OnceCell<HashMap<IpNetwork, InstanceInfo>> = OnceCell::const_new();

static CLIENT: OnceCell<Client> = OnceCell::const_new();

async fn get_client<'a>() -> &'a Client {
    CLIENT
        .get_or_init(|| async { Client::new(get_conf().await) })
        .await
}

pub async fn get_ip_map<'a>() -> &'a HashMap<IpNetwork, InstanceInfo> {
    IP_MAP
        .get_or_init(|| async {
            let client = get_client().await;
            let instances = client.describe_instances().send().await.unwrap();

            let mut map = HashMap::new();

            for res in instances.reservations().unwrap_or_default() {
                for instance in res.instances().unwrap_or_default() {
                    let name = instance.key_name().unwrap_or_default().to_string();
                    if let Ok(ip) =
                        IpNetwork::from_str(instance.public_ip_address().unwrap_or_default())
                    {
                        let groups = instance.security_groups().unwrap_or_default();
                        if !groups.is_empty() {
                            let idents = groups
                                .iter()
                                .flat_map(|g| g.group_id().map(|s| s.to_string()))
                                .collect();
                            map.insert(ip, InstanceInfo { name, idents });
                        }
                    }
                }
            }

            map
        })
        .await
}

pub async fn add_allow(allow_ip: IpNetwork, info: &InstanceInfo) -> Result<(), Error> {
    let client = get_client().await;
    let name = info.name.clone();

    let mut ip = allow_ip.to_string();
    ip.push_str("/32");

    for ident in info.idents.iter() {
        match client
            .authorize_security_group_ingress()
            .set_cidr_ip(Some(ip.to_string()))
            .set_to_port(Some(22))
            .set_ip_protocol(Some("tcp".to_string()))
            .set_group_id(Some(ident.clone()))
            .send()
            .await
        {
            Ok(_) => info!("Allow {ip} to {ident} for {name}"),
            Err(err) => error!("Err allowing {ip} to {ident} for {name}: {err}"),
        };
    }

    Ok(())
}