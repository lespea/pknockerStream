// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "inet_proto"))]
    pub struct InetProto;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::InetProto;

    blocks (id) {
        id -> Int4,
        src_ip -> Inet,
        dst_ip -> Inet,
        proto -> InetProto,
        event_ts -> Timestamptz,
        insert_ts -> Timestamptz,
    }
}
