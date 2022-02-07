use crate::services::ForwardedPaymentsFilter;
use crate::{hex_utils, utils};
use crate::{
    node::PaymentInfo,
    services::{PaginationRequest, PaginationResponse, PaymentsFilter},
};

use super::Error;
use rusqlite::{named_params, Connection};
use serde::Serialize;

#[derive(Debug, Serialize, PartialEq, Clone)]
pub struct ForwardedPayment {
    pub hours_since_epoch: u64,
    pub fees_earned_msat: u64,
    pub to_channel_id: Option<String>,
    pub from_channel_id: Option<String>,
    pub total_payments: u64,
}

#[derive(Debug, Serialize, PartialEq, Clone)]
pub struct Payment {
    pub id: i64,
    pub payment_hash: String,
    pub preimage: Option<String>,
    pub secret: Option<String>,
    pub status: String,
    pub origin: String,
    pub amt_msat: Option<u64>,
    pub created_at: String,
    pub updated_at: String,
    pub label: Option<String>,
    pub invoice: Option<String>,
}

impl From<PaymentInfo> for Payment {
    fn from(payment: PaymentInfo) -> Self {
        Self {
            id: 0,
            created_at: String::from(""),
            updated_at: String::from(""),
            label: payment.label,
            payment_hash: hex_utils::hex_str(&payment.hash.0),
            preimage: payment
                .preimage
                .map(|preimage| hex_utils::hex_str(&preimage.0)),
            secret: payment.secret.map(|secret| hex_utils::hex_str(&secret.0)),
            status: payment.status.to_string(),
            amt_msat: payment.amt_msat.0,
            origin: payment.origin.to_string(),
            invoice: payment.invoice,
        }
    }
}

static MIGRATIONS: &[&str] = &[
    "CREATE TABLE version (version INTEGER)",
    "INSERT INTO version VALUES (1)",
    "CREATE TABLE seed (seed BLOB)",
    "CREATE TABLE macaroons (identifier BLOB PRIMARY KEY)",
    "CREATE TABLE payments (id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, created_at INTEGER NOT NULL DEFAULT current_timestamp, updated_at INTEGER NOT NULL DEFAULT current_timestamp, payment_hash TEXT, preimage TEXT, secret TEXT, status TEXT, amt_msat INTEGER, origin TEXT, label TEXT, invoice TEXT)",
    "CREATE TRIGGER tg_payments_updated_at AFTER UPDATE ON payments FOR EACH ROW BEGIN UPDATE payments SET updated_at = current_timestamp WHERE id=old.id; END;",
    "CREATE UNIQUE INDEX idx_payment_hash ON payments(payment_hash)",
    "CREATE TABLE forwarded_payments (id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, created_at INTEGER NOT NULL DEFAULT current_timestamp, updated_at INTEGER NOT NULL DEFAULT current_timestamp, hours_since_epoch INTEGER NOT NULL, total_earned_msat INTEGER NOT NULL DEFAULT 0, total_payments INTEGER NOT NULL DEFAULT 1, from_channel_id TEXT, to_channel_id TEXT)",
    "CREATE TRIGGER tg_forwarded_payments_updated AFTER UPDATE ON forwarded_payments FOR EACH ROW BEGIN UPDATE forwarded_payments SET updated_at = current_timestamp, total_payments = old.total_payments + 1 WHERE id=old.id; END;",
    "CREATE INDEX idx_from_channel_id ON forwarded_payments(from_channel_id)",
    "CREATE INDEX idx_to_channel_id ON forwarded_payments(to_channel_id)",
    "CREATE UNIQUE INDEX idx_hours_since_epoch ON forwarded_payments(hours_since_epoch, from_channel_id, to_channel_id)"
    ];

pub struct NodeDatabase {
    pub path: String,
    pub connection: Connection,
}

impl NodeDatabase {
    pub fn new(path: String) -> Self {
        let connection = get_connection(&path).unwrap();
        Self { connection, path }
    }

    pub fn clone(&self) -> Self {
        Self::new(self.path.clone())
    }
}

impl NodeDatabase {
    pub fn create_seed(&mut self, seed: Vec<u8>) -> Result<(), Error> {
        let mut statement = self
            .connection
            .prepare_cached("INSERT INTO seed (seed) VALUES (:seed)")?;

        statement.execute(named_params! {
            ":seed": seed,
        })?;

        Ok(())
    }

    pub fn get_seed(&mut self) -> Result<Option<Vec<u8>>, Error> {
        let mut statement = self
            .connection
            .prepare_cached("SELECT seed.seed FROM seed")?;
        let mut rows = statement.query([])?;

        let row = rows.next()?;

        match row {
            Some(row) => {
                let seed: Vec<u8> = row.get(0)?;
                Ok(Some(seed))
            }
            None => Ok(None),
        }
    }

    pub fn create_macaroon(&mut self, identifier: Vec<u8>) -> Result<(), Error> {
        let mut statement = self
            .connection
            .prepare_cached("INSERT INTO macaroons (identifier) VALUES (:identifier)")?;

        statement.execute(named_params! {
            ":identifier": identifier,
        })?;

        Ok(())
    }

    pub fn macaroon_exists(&mut self, identifier: Vec<u8>) -> Result<bool, Error> {
        let mut statement = self.connection.prepare_cached(
            "SELECT macaroons.identifier FROM macaroons WHERE macaroons.identifier=:identifier",
        )?;
        let mut rows = statement.query(named_params! { ":identifier": identifier })?;
        let row = rows.next()?;
        Ok(row.is_some())
    }

    pub fn get_forwarded_payments(
        &mut self,
        filter: ForwardedPaymentsFilter,
    ) -> Result<Vec<ForwardedPayment>, Error> {
        let from_channel_id = filter.from_channel_id.unwrap_or_else(|| String::from(""));
        let to_channel_id = filter.to_channel_id.unwrap_or_else(|| String::from(""));
        let from_hours_since_epoch = filter.from_hours_since_epoch.unwrap_or(0);
        // TODO: should we just always use u64::MAX since there's no difference and it can't fail?
        let hours_since_epoch = utils::hours_since_epoch().unwrap_or(u64::MAX);
        let to_hours_since_epoch = filter.to_hours_since_epoch.unwrap_or(hours_since_epoch);

        let mut statement = self.connection.prepare_cached("
            SELECT forwarded_payments.total_payments, forwarded_payments.total_earned_msat, forwarded_payments.hours_since_epoch, forwarded_payments.from_channel_id, forwarded_payments.to_channel_id 
            FROM forwarded_payments
            WHERE instr(forwarded_payments.from_channel_id, :from_channel_id) > 0 AND instr(forwarded_payments.to_channel_id, :to_channel_id) > 0 AND forwarded_payments.hours_since_epoch >= :from_hours_since_epoch AND forwarded_payments.hours_since_epoch <= :to_hours_since_epoch
            ORDER BY hours_since_epoch ASC
        ")?;

        let forwarded_payments: Vec<ForwardedPayment> = statement
            .query_map(
                named_params! {
                    ":from_channel_id": from_channel_id,
                    ":to_channel_id": to_channel_id,
                    ":from_hours_since_epoch": from_hours_since_epoch,
                    ":to_hours_since_epoch": to_hours_since_epoch
                },
                |row| {
                    Ok(ForwardedPayment {
                        total_payments: row.get(0)?,
                        fees_earned_msat: row.get(1)?,
                        hours_since_epoch: row.get(2)?,
                        from_channel_id: row.get(3)?,
                        to_channel_id: row.get(4)?,
                    })
                },
            )?
            .flatten()
            .collect();

        Ok(forwarded_payments)
    }

    pub fn record_forwarded_payment(
        &mut self,
        forwarded_payment: ForwardedPayment,
    ) -> Result<(), Error> {
        let mut statement = self.connection.prepare_cached("
            INSERT INTO forwarded_payments (hours_since_epoch, to_channel_id, from_channel_id, total_earned_msat) 
            VALUES (:hours_since_epoch, :to_channel_id, :from_channel_id, :fees_earned_msat) 
            ON CONFLICT
            DO UPDATE SET total_earned_msat = total_earned_msat + excluded.total_earned_msat
        ")?;

        statement.execute(named_params! {
            ":hours_since_epoch": forwarded_payment.hours_since_epoch,
            ":to_channel_id": forwarded_payment.to_channel_id,
            ":from_channel_id": forwarded_payment.from_channel_id,
            ":fees_earned_msat": forwarded_payment.fees_earned_msat,
        })?;

        Ok(())
    }

    pub fn create_or_update_payment(&mut self, payment: Payment) -> Result<(), Error> {
        let mut statement = self.connection.prepare_cached("
            INSERT INTO payments (payment_hash, preimage, secret, status, amt_msat, origin, label, invoice) 
            VALUES (:payment_hash, :preimage, :secret, :status, :amt_msat, :origin, :label, :invoice) 
            ON CONFLICT(payment_hash) 
            DO UPDATE SET preimage=excluded.preimage, secret=excluded.secret, status=excluded.status, amt_msat=excluded.amt_msat, origin=excluded.origin, label=excluded.label, invoice=excluded.invoice
        ")?;

        statement.execute(named_params! {
            ":payment_hash": payment.payment_hash,
            ":preimage": payment.preimage,
            ":secret": payment.secret,
            ":status": payment.status,
            ":amt_msat": payment.amt_msat,
            ":origin": payment.origin,
            ":label": payment.label,
            ":invoice": payment.invoice
        })?;

        Ok(())
    }

    pub fn delete_payment(&self, payment_hash: String) -> Result<(), Error> {
        let mut statement = self
            .connection
            .prepare_cached("DELETE FROM payments WHERE payments.payment_hash = :payment_hash")?;
        statement.execute(named_params! { ":payment_hash": payment_hash})?;
        Ok(())
    }

    pub fn get_payment(&self, payment_hash: String) -> Result<Option<Payment>, Error> {
        let mut statement = self
            .connection
            .prepare_cached("SELECT payments.id, payments.created_at, payments.updated_at, payments.payment_hash, payments.preimage, payments.secret, payments.status, payments.amt_msat, payments.origin, payments.label, payments.invoice FROM payments WHERE payment_hash=:payment_hash")?;
        let mut rows = statement.query(named_params! { ":payment_hash": payment_hash })?;

        let row = rows.next()?;

        match row {
            Some(row) => Ok(Some(Payment {
                id: row.get(0)?,
                created_at: row.get(1)?,
                updated_at: row.get(2)?,
                payment_hash: row.get(3)?,
                preimage: row.get(4)?,
                secret: row.get(5)?,
                status: row.get(6)?,
                amt_msat: row.get(7)?,
                origin: row.get(8)?,
                label: row.get(9)?,
                invoice: row.get(10)?,
            })),
            None => Ok(None),
        }
    }

    pub fn get_payments(
        &self,
        pagination: PaginationRequest,
        filter: PaymentsFilter,
    ) -> Result<(Vec<Payment>, PaginationResponse), Error> {
        let origin_filter = filter.origin.unwrap_or_else(|| String::from(""));
        let status_filter = filter.status.unwrap_or_else(|| String::from(""));
        let query_string = pagination.query.unwrap_or_else(|| String::from(""));

        let mut count_statement = self
            .connection
            .prepare("SELECT COUNT(1) as cnt FROM payments WHERE instr(payments.origin, :origin) > 0 AND instr(payments.status, :status) > 0 AND (instr(payments.invoice, :query) > 0 OR instr(payments.payment_hash, :query) > 0)")?;

        let count = count_statement.query_row(
            named_params! {
                ":origin": origin_filter,
                ":status": status_filter,
                ":query": query_string
            },
            |row| {
                let count = row.get(0).unwrap_or(0);
                Ok(count as u64)
            },
        )?;

        let mut statement = self.connection.prepare_cached("
            SELECT payments.id, payments.created_at, payments.updated_at, payments.payment_hash, payments.preimage, payments.secret, payments.status, payments.amt_msat, payments.origin, payments.label, payments.invoice 
            FROM payments
            WHERE instr(payments.origin, :origin) > 0 AND instr(payments.status, :status) > 0 AND (instr(payments.invoice, :query) > 0 OR instr(payments.payment_hash, :query) > 0)
            ORDER BY updated_at DESC
            LIMIT :take
            OFFSET :offset
        ")?;

        let mut payments: Vec<Payment> = statement
            .query_map(
                named_params! {
                    ":offset": pagination.page * pagination.take,
                    ":take": pagination.take + 1,
                    ":origin": origin_filter,
                    ":status": status_filter,
                    ":query": query_string
                },
                |row| {
                    Ok(Payment {
                        id: row.get(0)?,
                        created_at: row.get(1)?,
                        updated_at: row.get(2)?,
                        payment_hash: row.get(3)?,
                        preimage: row.get(4)?,
                        secret: row.get(5)?,
                        status: row.get(6)?,
                        amt_msat: row.get(7)?,
                        origin: row.get(8)?,
                        label: row.get(9)?,
                        invoice: row.get(10)?,
                    })
                },
            )?
            .flatten()
            .collect();

        let has_more = payments.len() > pagination.take as usize;

        if has_more {
            payments.pop();
        }
        let pagination = PaginationResponse {
            has_more,
            total: count,
        };
        Ok((payments, pagination))
    }
}

pub fn get_connection(path: &str) -> Result<Connection, rusqlite::Error> {
    let connection = Connection::open(path)?;
    migrate(&connection)?;
    Ok(connection)
}

fn get_schema_version(conn: &Connection) -> rusqlite::Result<i32> {
    let statement = conn.prepare_cached("SELECT version FROM version");
    match statement {
        Err(rusqlite::Error::SqliteFailure(e, Some(msg))) => {
            if msg == "no such table: version" {
                Ok(0)
            } else {
                Err(rusqlite::Error::SqliteFailure(e, Some(msg)))
            }
        }
        Ok(mut stmt) => {
            let mut rows = stmt.query([])?;
            match rows.next()? {
                Some(row) => {
                    let version: i32 = row.get(0)?;
                    Ok(version)
                }
                None => Ok(0),
            }
        }
        _ => Ok(0),
    }
}

fn set_schema_version(conn: &Connection, version: i32) -> rusqlite::Result<usize> {
    conn.execute(
        "UPDATE version SET version=:version",
        named_params! {":version": version},
    )
}

fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    let version = get_schema_version(conn)?;
    let stmts = &MIGRATIONS[(version as usize)..];
    let mut i: i32 = version;

    if version == MIGRATIONS.len() as i32 {
        return Ok(());
    }

    println!(
        "migrating db from version {} to {}",
        version,
        MIGRATIONS.len()
    );
    for stmt in stmts {
        let res = conn.execute(stmt, []);
        if res.is_err() {
            println!("migration failed on:\n{}\n{:?}", stmt, res);
            break;
        }

        i += 1;
    }

    set_schema_version(conn, i)?;

    Ok(())
}
