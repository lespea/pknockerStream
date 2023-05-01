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
        port -> Int4,
        event_ts -> Timestamptz,
        insert_ts -> Timestamptz,
    }
}

diesel::table! {
    denies (ip) {
        ip -> Inet,
    }
}

diesel::allow_tables_to_appear_in_same_query!(blocks, denies,);
