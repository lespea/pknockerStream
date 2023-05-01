use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{Integer, Timestamptz};
use ipnetwork::IpNetwork;

#[derive(diesel_derive_enum::DbEnum, Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[ExistingTypePath = "crate::schema::sql_types::InetProto"]
pub enum InetProto {
    Tcp,
    Udp,
    Icmp,
}

#[derive(Queryable, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[diesel(table_name = crate::schema::blocks)]
pub struct Block {
    pub id: i32,
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
}
