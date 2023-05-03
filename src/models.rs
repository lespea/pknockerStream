use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::prelude::*;
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct Conns(pub Vec<(InetProto, u16)>);

#[derive(
    diesel_derive_enum::DbEnum,
    Debug,
    Copy,
    Clone,
    Eq,
    PartialEq,
    Hash,
    Ord,
    PartialOrd,
    Deserialize,
    Serialize,
)]
#[ExistingTypePath = "crate::schema::sql_types::InetProto"]
#[serde(rename_all = "lowercase")]
pub enum InetProto {
    Tcp,
    Udp,
    Icmp,
}

#[derive(Queryable, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[diesel(table_name = crate::schema::blocks)]
pub struct Block {
    pub id: i64,
    pub src_ip: IpNetwork,
    pub dst_ip: IpNetwork,
    pub proto: InetProto,
    pub port: i32,
    pub event_ts: NaiveDateTime,
    pub insert_ts: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::blocks)]
pub struct NewBlock {
    pub src_ip: IpNetwork,
    pub dst_ip: IpNetwork,
    pub proto: InetProto,
    pub port: i32,
    pub event_ts: DateTime<Utc>,
}

#[derive(Queryable, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[diesel(table_name = crate::schema::denies)]
pub struct Denies {
    pub ip: IpNetwork,
    pub added_on: DateTime<Utc>,
}

#[derive(Queryable, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[diesel(table_name = crate::schema::denies)]
pub struct ViewToCheck {
    pub src_ip: IpNetwork,
    pub dst_ip: IpNetwork,
    pub conns: String,
}

table! {
    use diesel::sql_types::*;
    use crate::schema::sql_types::InetProto;

    view_to_check (src_ip) {
        src_ip -> Inet,
        dst_ip -> Inet,
        conns -> Text,
    }
}
