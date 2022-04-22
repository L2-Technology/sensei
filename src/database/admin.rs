// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use super::Error;
use crate::utils::{self};
use crate::{
    hex_utils,
    services::{PaginationRequest, PaginationResponse},
};
use rand::{thread_rng, Rng};
use rusqlite::{named_params, Connection};
use serde::Serialize;
use uuid::Uuid;

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Self::Generic(e.to_string())
    }
}

pub enum Status {
    Stopped,
    Running,
}

impl Status {
    pub fn to_integer(&self) -> u8 {
        match self {
            Status::Stopped => 0,
            Status::Running => 1,
        }
    }
}

pub enum Role {
    Admin,
    User,
}

impl Role {
    pub fn to_integer(&self) -> u8 {
        match self {
            Role::Admin => 0,
            Role::User => 1,
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Clone)]
pub struct Node {
    pub id: i64,
    pub external_id: String,
    pub role: u8,
    pub username: String,
    pub alias: String,
    pub network: String,
    pub listen_addr: String,
    pub listen_port: u16,
    pub pubkey: String,
    pub created_at: String,
    pub updated_at: String,
    pub status: u8,
}

impl Node {
    pub fn new(
        username: String,
        alias: String,
        network: String,
        listen_addr: String,
        listen_port: u16,
    ) -> Self {
        Self {
            id: 0,
            external_id: Uuid::new_v4().to_string(),
            role: Role::User.to_integer(),
            username,
            alias,
            network,
            listen_addr,
            listen_port,
            pubkey: "".to_string(),
            created_at: "".to_string(),
            updated_at: "".to_string(),
            status: Status::Stopped.to_integer(),
        }
    }

    pub fn new_admin(
        username: String,
        alias: String,
        network: String,
        listen_addr: String,
        listen_port: u16,
    ) -> Self {
        Self {
            id: 0,
            external_id: Uuid::new_v4().to_string(),
            role: Role::Admin.to_integer(),
            username,
            alias,
            network,
            listen_addr,
            listen_port,
            pubkey: "".to_string(),
            created_at: "".to_string(),
            updated_at: "".to_string(),
            status: Status::Stopped.to_integer(),
        }
    }

    pub fn is_admin(&self) -> bool {
        self.role == Role::Admin.to_integer()
    }

    pub fn is_user(&self) -> bool {
        self.role == Role::User.to_integer()
    }
}

#[derive(Debug, Serialize, PartialEq, Clone)]
pub struct AccessToken {
    pub id: u64,
    pub name: String,
    pub external_id: String,
    pub scope: String,
    pub token: String,
    pub expires_at: u64,
    pub single_use: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl AccessToken {
    pub fn new(name: String, scope: String, expires_at: u64, single_use: bool) -> Self {
        let mut token_bytes: [u8; 32] = [0; 32];
        thread_rng().fill_bytes(&mut token_bytes);
        let token = hex_utils::hex_str(&token_bytes);
        Self {
            id: 0,
            name,
            external_id: Uuid::new_v4().to_string(),
            token,
            scope,
            expires_at,
            single_use,
            created_at: "".to_string(),
            updated_at: "".to_string(),
        }
    }

    pub fn new_admin() -> Self {
        Self::new("admin".to_string(), "*".to_string(), 0, false)
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at != 0 && utils::seconds_since_epoch().unwrap() > self.expires_at
    }

    pub fn has_access_to_scope(&self, scope: Option<&str>) -> bool {
        match scope {
            Some(scope) => {
                let scopes: Vec<&str> = self.scope.split(',').collect();
                scopes.contains(&"*") || scopes.contains(&scope)
            }
            None => true,
        }
    }

    pub fn is_valid(&self, scope: Option<&str>) -> bool {
        !self.is_expired() && self.has_access_to_scope(scope)
    }
}

static MIGRATIONS: &[&str] = &[
    "CREATE TABLE version (version INTEGER)",
    "INSERT INTO version VALUES (1)",
    "CREATE TABLE nodes (id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, external_id TEXT NOT NULL, role INTEGER, username TEXT, alias TEXT, network TEXT, listen_addr TEXT, listen_port INTEGER, pubkey TEXT, status INTEGER DEFAULT 0, created_at INTEGER NOT NULL DEFAULT current_timestamp, updated_at INTEGER NOT NULL DEFAULT current_timestamp)",
    "CREATE TRIGGER tg_nodes_updated_at AFTER UPDATE ON nodes FOR EACH ROW BEGIN UPDATE nodes SET updated_at = current_timestamp WHERE id=old.id; END;",
    "CREATE UNIQUE INDEX idx_external_id ON nodes(external_id)",
    "CREATE UNIQUE INDEX idx_pubkey ON nodes(pubkey)",
    "CREATE INDEX idx_role ON nodes(role)",
    "CREATE TABLE access_tokens (id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, external_id TEXT NOT NULL, name TEXT, token TEXT, scope TEXT, expires_at INTEGER DEFAULT 0, single_use INTEGER DEFAULT 0, created_at INTEGER NOT NULL DEFAULT current_timestamp, updated_at INTEGER NOT NULL DEFAULT current_timestamp)"
];

pub struct AdminDatabase {
    pub path: String,
    pub connection: Connection,
}

impl AdminDatabase {
    pub fn new(path: String) -> Self {
        let connection = get_connection(&path).unwrap();
        Self { connection, path }
    }

    pub fn clone(&self) -> Self {
        Self::new(self.path.clone())
    }
}

impl AdminDatabase {
    pub fn create_access_token(&mut self, access_token: &AccessToken) -> Result<i64, Error> {
        let mut statement = self.connection.prepare_cached("INSERT INTO access_tokens (external_id, name, token, scope, expires_at, single_use) VALUES (:external_id, :name, :token, :scope, :expires_at, :single_use)")?;

        statement.execute(named_params! {
            ":external_id": access_token.external_id,
            ":name": access_token.name,
            ":token": access_token.token,
            ":scope": access_token.scope,
            ":expires_at": access_token.expires_at,
            ":single_use": access_token.single_use
        })?;

        Ok(self.connection.last_insert_rowid())
    }

    pub fn delete_access_token(&mut self, id: u64) -> Result<(), Error> {
        let mut statement = self
            .connection
            .prepare_cached("DELETE FROM access_tokens WHERE id=:id")?;
        statement.execute(named_params! { ":id": id})?;
        Ok(())
    }

    pub fn get_admin_access_token(&mut self) -> Result<Option<AccessToken>, Error> {
        self.get_access_token_by_scope("*".to_string())
    }

    pub fn get_access_token_by_scope(
        &mut self,
        scope: String,
    ) -> Result<Option<AccessToken>, Error> {
        let mut statement = self.connection.prepare_cached(
            "SELECT id, external_id, name, created_at, updated_at, token, scope, expires_at, single_use FROM access_tokens WHERE scope=:scope"
        )?;

        let mut rows = statement.query(named_params! { ":scope": scope })?;

        match rows.next()? {
            Some(row) => Ok(Some(AccessToken {
                id: row.get(0)?,
                external_id: row.get(1)?,
                name: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
                token: row.get(5)?,
                scope: row.get(6)?,
                expires_at: row.get(7)?,
                single_use: row.get(8)?,
            })),
            None => Ok(None),
        }
    }

    pub fn get_access_token(&mut self, token: String) -> Result<Option<AccessToken>, Error> {
        let mut statement = self.connection.prepare_cached(
            "SELECT id, external_id, name, created_at, updated_at, token, scope, expires_at, single_use FROM access_tokens WHERE token=:token",
        )?;

        let mut rows = statement.query(named_params! { ":token": token })?;

        match rows.next()? {
            Some(row) => Ok(Some(AccessToken {
                id: row.get(0)?,
                external_id: row.get(1)?,
                name: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
                token: row.get(5)?,
                scope: row.get(6)?,
                expires_at: row.get(7)?,
                single_use: row.get(8)?,
            })),
            None => Ok(None),
        }
    }

    pub fn list_access_tokens(
        &mut self,
        pagination: PaginationRequest,
    ) -> Result<(Vec<AccessToken>, PaginationResponse), Error> {
        let query_string = pagination.query.unwrap_or_else(|| String::from(""));

        let mut count_statement = self
            .connection
            .prepare("SELECT COUNT(1) as cnt FROM access_tokens WHERE instr(access_tokens.token, :query) > 0")?;

        let count = count_statement.query_row(
            named_params! {
                ":query": query_string
            },
            |row| {
                let count = row.get(0).unwrap_or(0);
                Ok(count as u64)
            },
        )?;

        let mut statement = self.connection.prepare(
            "
            SELECT id, external_id, name, created_at, updated_at, token, scope, expires_at, single_use
            FROM access_tokens
            WHERE instr(access_tokens.token, :query) > 0
            ORDER BY access_tokens.updated_at DESC
            LIMIT :take
            OFFSET :offset
        ",
        )?;
        let mut rows = statement.query(named_params! {
            ":offset": pagination.page * pagination.take,
            ":take": pagination.take + 1,
            ":query": query_string
        })?;

        let mut tokens = Vec::new();
        while let Some(row) = rows.next()? {
            tokens.push(AccessToken {
                id: row.get(0)?,
                external_id: row.get(1)?,
                name: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
                token: row.get(5)?,
                scope: row.get(6)?,
                expires_at: row.get(7)?,
                single_use: row.get(8)?,
            })
        }

        let has_more = tokens.len() > pagination.take as usize;

        if has_more {
            tokens.pop();
        }
        let pagination = PaginationResponse {
            has_more,
            total: count,
        };
        Ok((tokens, pagination))
    }

    pub fn create_node(&mut self, node: Node) -> Result<i64, Error> {
        let mut statement = self.connection.prepare_cached("INSERT INTO nodes (external_id, username, alias, role, network, listen_addr, listen_port, pubkey, status) VALUES (:external_id, :username, :alias, :role, :network, :listen_addr, :listen_port, :pubkey, :status)")?;

        statement.execute(named_params! {
            ":external_id": node.external_id,
            ":username": node.username,
            ":alias": node.alias,
            ":role": node.role,
            ":network": node.network,
            ":listen_addr": node.listen_addr,
            ":listen_port": node.listen_port,
            ":pubkey": node.pubkey,
            ":status": node.status
        })?;

        Ok(self.connection.last_insert_rowid())
    }

    pub fn update_node(&mut self, node: Node) -> Result<(), Error> {
        let mut statement = self.connection.prepare_cached("UPDATE nodes SET username=:username, alias=:alias, listen_addr=:listen_addr, listen_port=:listen_port, pubkey=:pubkey, status=:status WHERE id=:id")?;

        statement.execute(named_params! {
            ":id": node.id,
            ":username": node.username,
            ":alias": node.alias,
            ":listen_addr": node.listen_addr,
            ":listen_port": node.listen_port,
            ":pubkey": node.pubkey,
            ":status": node.status
        })?;

        Ok(())
    }

    pub fn delete_node(&mut self, node_id: i64) -> Result<(), Error> {
        let mut statement = self
            .connection
            .prepare_cached("DELETE FROM nodes WHERE id=:id")?;
        statement.execute(named_params! { ":id": node_id})?;
        Ok(())
    }

    pub fn mark_all_nodes_stopped(&mut self) -> Result<(), Error> {
        let mut statement = self
            .connection
            .prepare_cached("UPDATE nodes SET status=0")?;
        statement.execute([])?;
        Ok(())
    }

    pub fn list_nodes(
        &mut self,
        pagination: PaginationRequest,
    ) -> Result<(Vec<Node>, PaginationResponse), Error> {
        let query_string = pagination.query.unwrap_or_else(|| String::from(""));

        let mut count_statement = self
            .connection
            .prepare("SELECT COUNT(1) as cnt FROM nodes WHERE (instr(nodes.alias, :query) > 0 OR instr(nodes.username, :query) > 0 OR instr(nodes.pubkey, :query) > 0)")?;

        let count = count_statement.query_row(
            named_params! {
                ":query": query_string
            },
            |row| {
                let count = row.get(0).unwrap_or(0);
                Ok(count as u64)
            },
        )?;

        let mut statement = self.connection.prepare("
            SELECT nodes.id, nodes.external_id, nodes.created_at, nodes.updated_at, nodes.username, nodes.alias, nodes.role, nodes.network, nodes.listen_addr, nodes.listen_port, nodes.pubkey, nodes.status
            FROM nodes
            WHERE (instr(nodes.alias, :query) > 0 OR instr(nodes.username, :query) > 0 OR instr(nodes.pubkey, :query) > 0)
            ORDER BY nodes.updated_at DESC
            LIMIT :take
            OFFSET :offset
        ")?;
        let mut rows = statement.query(named_params! {
            ":offset": pagination.page * pagination.take,
            ":take": pagination.take + 1,
            ":query": query_string
        })?;

        let mut nodes = Vec::new();
        while let Some(row) = rows.next()? {
            nodes.push(Node {
                id: row.get(0)?,
                external_id: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
                username: row.get(4)?,
                alias: row.get(5)?,
                role: row.get(6)?,
                network: row.get(7)?,
                listen_addr: row.get(8)?,
                listen_port: row.get(9)?,
                pubkey: row.get(10)?,
                status: row.get(11)?,
            })
        }

        let has_more = nodes.len() > pagination.take as usize;

        if has_more {
            nodes.pop();
        }
        let pagination = PaginationResponse {
            has_more,
            total: count,
        };
        Ok((nodes, pagination))
    }

    pub fn get_node(&mut self, id: i64) -> Result<Option<Node>, Error> {
        let mut statement = self.connection.prepare_cached(
            "SELECT id, external_id, created_at, updated_at, username, alias, role, network, listen_addr, listen_port, pubkey, status FROM nodes WHERE id=:id",
        )?;

        let mut rows = statement.query(named_params! { ":id": id })?;

        match rows.next()? {
            Some(row) => Ok(Some(Node {
                id: row.get(0)?,
                external_id: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
                username: row.get(4)?,
                alias: row.get(5)?,
                role: row.get(6)?,
                network: row.get(7)?,
                listen_addr: row.get(8)?,
                listen_port: row.get(9)?,
                pubkey: row.get(10)?,
                status: row.get(11)?,
            })),
            None => Ok(None),
        }
    }

    pub fn get_admin_node(&mut self) -> Result<Option<Node>, Error> {
        let mut statement = self.connection.prepare_cached(
            "SELECT id, external_id, created_at, updated_at, username, alias, role, network, listen_addr, listen_port, pubkey, status FROM nodes WHERE role=:role",
        )?;

        let mut rows = statement.query(named_params! { ":role": Role::Admin.to_integer() })?;

        match rows.next()? {
            Some(row) => Ok(Some(Node {
                id: row.get(0)?,
                external_id: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
                username: row.get(4)?,
                alias: row.get(5)?,
                role: row.get(6)?,
                network: row.get(7)?,
                listen_addr: row.get(8)?,
                listen_port: row.get(9)?,
                pubkey: row.get(10)?,
                status: row.get(11)?,
            })),
            None => Ok(None),
        }
    }

    pub fn get_node_by_pubkey(&mut self, pubkey: &str) -> Result<Option<Node>, Error> {
        let mut statement = self.connection.prepare_cached(
            "SELECT id, external_id, created_at, updated_at, username, alias, role, network, listen_addr, listen_port, status, pubkey FROM nodes WHERE pubkey=:pubkey",
        )?;

        let mut rows = statement.query(named_params! { ":pubkey": pubkey })?;

        match rows.next()? {
            Some(row) => Ok(Some(Node {
                id: row.get(0)?,
                external_id: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
                username: row.get(4)?,
                alias: row.get(5)?,
                role: row.get(6)?,
                network: row.get(7)?,
                listen_addr: row.get(8)?,
                listen_port: row.get(9)?,
                status: row.get(10)?,
                pubkey: row.get(11)?,
            })),
            None => Ok(None),
        }
    }

    pub fn get_node_by_username(&mut self, username: String) -> Result<Option<Node>, Error> {
        let mut statement = self.connection.prepare_cached(
            "SELECT id, external_id, created_at, updated_at, username, alias, role, network, listen_addr, listen_port, status, pubkey FROM nodes WHERE username=:username",
        )?;

        let mut rows = statement.query(named_params! { ":username": username })?;

        match rows.next()? {
            Some(row) => Ok(Some(Node {
                id: row.get(0)?,
                external_id: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
                username: row.get(4)?,
                alias: row.get(5)?,
                role: row.get(6)?,
                network: row.get(7)?,
                listen_addr: row.get(8)?,
                listen_port: row.get(9)?,
                status: row.get(10)?,
                pubkey: row.get(11)?,
            })),
            None => Ok(None),
        }
    }

    pub fn port_in_use(&mut self, port: u16) -> Result<bool, Error> {
        let mut statement = self
            .connection
            .prepare_cached("SELECT * FROM nodes WHERE listen_port=:listen_port")?;
        let mut rows = statement.query(named_params! { ":listen_port": port })?;
        let row = rows.next()?;
        Ok(row.is_some())
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
