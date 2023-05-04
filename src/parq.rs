use std::str::FromStr;

use bytes::Bytes;
use chrono::{TimeZone, Utc};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use ipnetwork::IpNetwork;
use lambda_runtime::Error;
use parquet::arrow::ProjectionMask;
use parquet::file::metadata::ParquetMetaData;
use parquet::file::reader::FileReader;
use parquet::file::reader::SerializedFileReader;
use parquet::record::{Row, RowAccessor};
use tracing::log::{debug, error, info};

use crate::models::{InetProto, NewBlock};
use crate::schema::blocks;

pub async fn add_records(
    data: Vec<u8>,
    pool: &Pool<AsyncPgConnection>,
    add: bool,
) -> Result<(), Error> {
    let reader = SerializedFileReader::new(Bytes::from(data))?;
    let mut conn = pool.get().await?;

    let (fields, _) = Fields::from_metadata(reader.metadata());

    let mut to_add = Vec::with_capacity(reader.num_row_groups());
    let rows = reader.get_row_iter(None)?;

    for row in rows {
        match fields.to_block(row) {
            Err(e) => debug!("Error adding row - {e}"),

            Ok(block) => {
                if !add {
                    info!("Would add {to_add:?}");
                } else {
                    to_add.push(block);
                }
            }
        };
    }

    if !to_add.is_empty() {
        let res = diesel::insert_into(blocks::table)
            .values(&to_add)
            .execute(&mut conn)
            .await;

        if res.is_ok() {
            info!("Added {to_add:?}")
        } else {
            error!("Couldn't add {to_add:?}: {res:?}")
        }
    }

    Ok(())
}

#[derive(Default)]
struct WantFields {
    src: Option<usize>,
    dst: Option<usize>,
    port: Option<usize>,
    proto: Option<usize>,
    start: Option<usize>,
    action: Option<usize>,
}

impl WantFields {
    fn field(&mut self, field_name: &str, idx: usize) {
        if let Some(field) = match field_name {
            "srcaddr" => Some(&mut self.src),
            "dstaddr" => Some(&mut self.dst),
            "dstport" => Some(&mut self.port),
            "protocol" => Some(&mut self.proto),
            "start" => Some(&mut self.start),
            "action" => Some(&mut self.action),
            _ => None,
        } {
            if let Some(old) = field.replace(idx) {
                panic!("Dupe idx for {field_name} ({idx}/{old}");
            }
        }
    }

    fn build(self) -> Fields {
        Fields {
            src: self.src.unwrap(),
            dst: self.dst.unwrap(),
            port: self.port.unwrap(),
            proto: self.proto.unwrap(),
            start: self.start.unwrap(),
            action: self.action.unwrap(),
        }
    }
}

struct Fields {
    src: usize,
    dst: usize,
    port: usize,
    proto: usize,
    start: usize,
    action: usize,
}

impl Fields {
    fn from_metadata(metadata: &ParquetMetaData) -> (Fields, ProjectionMask) {
        let file_metadata = metadata.file_metadata();
        let mut want_fields = WantFields::default();
        for (idx, field) in file_metadata.schema_descr().columns().iter().enumerate() {
            want_fields.field(field.name(), idx);
        }

        let fields = want_fields.build();
        let mask = ProjectionMask::roots(file_metadata.schema_descr(), fields.all());

        (fields, mask)
    }

    fn all(&self) -> [usize; 6] {
        [
            self.src,
            self.dst,
            self.port,
            self.proto,
            self.start,
            self.action,
        ]
    }

    fn to_block(&self, row: Row) -> Result<NewBlock, Error> {
        match row.get_string(self.action) {
            Ok(s) if s == "REJECT" => (),
            Ok(s) => return Err(Error::from(format!("non-block entry ({s})"))),
            _ => (),
        };

        let proto = match row.get_int(self.proto) {
            Ok(6) => InetProto::Tcp,
            Ok(17) => InetProto::Udp,
            Ok(n) => return Err(Error::from(format!("Unknown proto number {n}"))),
            Err(e) => return Err(Error::from(e)),
        };

        Ok(NewBlock {
            src_ip: IpNetwork::from_str(row.get_string(self.src)?)?,
            dst_ip: IpNetwork::from_str(row.get_string(self.dst)?)?,
            proto,
            port: row.get_int(self.port)?,
            event_ts: match Utc.timestamp_opt(row.get_long(self.start)?, 0).single() {
                Some(ts) => ts,
                None => return Err(Error::from("invalid timestamp?")),
            },
        })
    }
}
