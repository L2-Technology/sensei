use crate::error::Error;
use crate::hex_utils;
use crate::services::PaginationRequest;
use crate::services::PaginationResponse;
use crate::services::PaymentsFilter;
use bitcoin::consensus::encode::{deserialize, serialize};
use bitcoin::BlockHash;
use entity::access_token;
use entity::access_token::Entity as AccessToken;
use entity::kv_store;
use entity::kv_store::Entity as KVStore;
use entity::macaroon;
use entity::macaroon::Entity as Macaroon;
use entity::node;
use entity::node::Entity as Node;
use entity::payment;
use entity::payment::Entity as Payment;
use entity::sea_orm;
use entity::sea_orm::ActiveValue;
use entity::sea_orm::QueryOrder;
use migration::Condition;
use migration::Expr;
use rand::thread_rng;
use rand::Rng;
use sea_orm::entity::EntityTrait;
use sea_orm::{prelude::*, DatabaseConnection};

pub struct SenseiDatabase {
    connection: DatabaseConnection,
    runtime_handle: tokio::runtime::Handle,
}

impl SenseiDatabase {
    pub fn new(connection: DatabaseConnection, runtime_handle: tokio::runtime::Handle) -> Self {
        Self {
            connection,
            runtime_handle,
        }
    }

    pub fn get_connection(&self) -> &DatabaseConnection {
        &self.connection
    }

    pub fn get_handle(&self) -> tokio::runtime::Handle {
        self.runtime_handle.clone()
    }

    pub async fn mark_all_nodes_stopped(&self) -> Result<(), Error> {
        Ok(Node::update_many()
            .col_expr(node::Column::Status, Expr::value(0))
            .exec(&self.connection)
            .await
            .map(|_| ())?)
    }

    pub async fn get_root_node(&self) -> Result<Option<node::Model>, Error> {
        Ok(Node::find()
            .filter(node::Column::Role.eq(node::NodeRole::Root))
            .one(&self.connection)
            .await?)
    }

    pub async fn get_node_by_pubkey(&self, pubkey: &str) -> Result<Option<node::Model>, Error> {
        Ok(Node::find()
            .filter(node::Column::Pubkey.eq(pubkey))
            .one(&self.connection)
            .await?)
    }

    pub async fn get_node_by_username(&self, username: &str) -> Result<Option<node::Model>, Error> {
        Ok(Node::find()
            .filter(node::Column::Username.eq(username))
            .one(&self.connection)
            .await?)
    }

    pub async fn get_node_by_connection_info(
        &self,
        listen_addr: &str,
        listen_port: i32,
    ) -> Result<Option<node::Model>, Error> {
        Ok(Node::find()
            .filter(node::Column::ListenAddr.eq(listen_addr))
            .filter(node::Column::ListenPort.eq(listen_port))
            .one(&self.connection)
            .await?)
    }

    pub async fn list_nodes(
        &self,
        pagination: PaginationRequest,
    ) -> Result<(Vec<node::Model>, PaginationResponse), Error> {
        let query_string = pagination.query.unwrap_or_else(|| String::from(""));
        let page_size: usize = pagination.take.try_into().unwrap();
        let page: usize = pagination.page.try_into().unwrap();

        let node_pages = Node::find()
            .filter(
                Condition::any()
                    .add(node::Column::Alias.contains(&query_string))
                    .add(node::Column::Pubkey.contains(&query_string))
                    .add(node::Column::Username.contains(&query_string)),
            )
            .order_by_desc(node::Column::UpdatedAt)
            .paginate(&self.connection, page_size);

        let nodes = node_pages.fetch_page(page).await?;
        let total = node_pages.num_items().await?;
        let has_more = ((page + 1) * page_size) < total;

        Ok((
            nodes,
            PaginationResponse {
                has_more,
                total: total.try_into().unwrap(),
            },
        ))
    }

    pub async fn get_root_access_token(&self) -> Result<Option<access_token::Model>, Error> {
        Ok(AccessToken::find()
            .filter(access_token::Column::Scope.eq(String::from("*")))
            .one(&self.connection)
            .await?)
    }

    pub async fn get_access_token_by_token(
        &self,
        token: String,
    ) -> Result<Option<access_token::Model>, Error> {
        Ok(AccessToken::find()
            .filter(access_token::Column::Token.eq(token))
            .one(&self.connection)
            .await?)
    }

    pub async fn create_root_access_token(&self) -> Result<access_token::Model, Error> {
        self.create_access_token("root".to_string(), "*".to_string(), 0, false)
            .await
    }

    pub async fn create_access_token(
        &self,
        name: String,
        scope: String,
        expires_at: i64,
        single_use: bool,
    ) -> Result<access_token::Model, Error> {
        let mut token_bytes: [u8; 32] = [0; 32];
        thread_rng().fill_bytes(&mut token_bytes);
        let token = hex_utils::hex_str(&token_bytes);

        let access_token = access_token::ActiveModel {
            id: ActiveValue::Set(Uuid::new_v4().to_string()),
            created_at: ActiveValue::NotSet,
            updated_at: ActiveValue::NotSet,
            name: ActiveValue::Set(name),
            token: ActiveValue::Set(token),
            scope: ActiveValue::Set(scope),
            expires_at: ActiveValue::Set(expires_at),
            single_use: ActiveValue::Set(single_use),
        };

        Ok(access_token.insert(&self.connection).await?)
    }

    pub async fn delete_access_token(&self, id: String) -> Result<(), Error> {
        let _res = AccessToken::delete_by_id(id).exec(&self.connection).await?;
        Ok(())
    }

    pub async fn list_access_tokens(
        &self,
        pagination: PaginationRequest,
    ) -> Result<(Vec<access_token::Model>, PaginationResponse), Error> {
        let query_string = pagination.query.unwrap_or_else(|| String::from(""));
        let page_size: usize = pagination.take.try_into().unwrap();
        let page: usize = pagination.page.try_into().unwrap();

        let access_token_pages = AccessToken::find()
            .filter(
                Condition::any()
                    .add(access_token::Column::Token.contains(&query_string))
                    .add(access_token::Column::Name.contains(&query_string)),
            )
            .order_by_desc(access_token::Column::UpdatedAt)
            .paginate(&self.connection, page_size);

        let access_tokens = access_token_pages.fetch_page(page).await?;
        let total = access_token_pages.num_items().await?;
        let has_more = ((page + 1) * page_size) < total;

        Ok((
            access_tokens,
            PaginationResponse {
                has_more,
                total: total.try_into().unwrap(),
            },
        ))
    }

    pub fn find_payment_sync(
        &self,
        node_id: String,
        payment_hash: String,
    ) -> Result<Option<payment::Model>, Error> {
        tokio::task::block_in_place(move || {
            self.runtime_handle
                .block_on(async move { self.find_payment(node_id, payment_hash).await })
        })
    }

    pub fn insert_payment_sync(
        &self,
        payment: payment::ActiveModel,
    ) -> Result<payment::Model, Error> {
        tokio::task::block_in_place(move || {
            self.runtime_handle
                .block_on(async move { Ok(payment.insert(&self.connection).await?) })
        })
    }

    pub fn update_payment_sync(&self, payment: payment::ActiveModel) -> Result<(), Error> {
        tokio::task::block_in_place(move || {
            self.runtime_handle.block_on(async move {
                let _update = payment.update(&self.connection).await?;
                Ok(())
            })
        })
    }

    pub async fn find_payment(
        &self,
        node_id: String,
        payment_hash: String,
    ) -> Result<Option<payment::Model>, Error> {
        Ok(Payment::find()
            .filter(entity::payment::Column::NodeId.eq(node_id))
            .filter(entity::payment::Column::PaymentHash.eq(payment_hash))
            .one(&self.connection)
            .await?)
    }

    pub async fn delete_payment(&self, node_id: String, payment_hash: String) -> Result<(), Error> {
        match self.find_payment(node_id, payment_hash).await? {
            Some(payment) => {
                let _deleted = payment.delete(&self.connection).await?;
                Ok(())
            }
            None => Ok(()),
        }
    }

    pub async fn label_payment(
        &self,
        node_id: String,
        payment_hash: String,
        label: String,
    ) -> Result<(), Error> {
        match self.find_payment(node_id, payment_hash).await? {
            Some(payment) => {
                let mut payment: payment::ActiveModel = payment.into();
                payment.label = ActiveValue::Set(Some(label));
                payment.update(&self.connection).await?;
                Ok(())
            }
            None => Ok(()),
        }
    }

    pub async fn list_payments(
        &self,
        node_id: String,
        pagination: PaginationRequest,
        filter: PaymentsFilter,
    ) -> Result<(Vec<payment::Model>, PaginationResponse), Error> {
        let origin_filter = filter.origin.unwrap_or_else(|| String::from(""));
        let status_filter = filter.status.unwrap_or_else(|| String::from(""));
        let query_string = pagination.query.unwrap_or_else(|| String::from(""));
        let page_size: usize = pagination.take.try_into().unwrap();
        let page: usize = pagination.page.try_into().unwrap();

        let payment_pages = Payment::find()
            .filter(payment::Column::NodeId.eq(node_id))
            .filter(payment::Column::Origin.contains(&origin_filter))
            .filter(payment::Column::Status.contains(&status_filter))
            .filter(
                Condition::any()
                    .add(payment::Column::PaymentHash.contains(&query_string))
                    .add(payment::Column::Label.contains(&query_string))
                    .add(payment::Column::Invoice.contains(&query_string)),
            )
            .order_by_desc(payment::Column::UpdatedAt)
            .paginate(&self.connection, page_size);

        let payments = payment_pages.fetch_page(page).await?;
        let total = payment_pages.num_items().await?;
        let has_more = ((page + 1) * page_size) < total;

        Ok((
            payments,
            PaginationResponse {
                has_more,
                total: total.try_into().unwrap(),
            },
        ))
    }

    pub async fn port_in_use(&self, listen_addr: &str, listen_port: i32) -> Result<bool, Error> {
        self.get_node_by_connection_info(listen_addr, listen_port)
            .await
            .map(|node| node.is_some())
    }

    pub async fn get_value(
        &self,
        node_id: String,
        key: String,
    ) -> Result<Option<kv_store::Model>, Error> {
        Ok(KVStore::find()
            .filter(kv_store::Column::NodeId.eq(node_id))
            .filter(kv_store::Column::K.eq(key))
            .one(&self.connection)
            .await?)
    }

    pub fn get_value_sync(
        &self,
        node_id: String,
        key: String,
    ) -> Result<Option<kv_store::Model>, Error> {
        tokio::task::block_in_place(move || {
            self.runtime_handle
                .block_on(async move { self.get_value(node_id, key).await })
        })
    }

    pub async fn list_values(
        &self,
        node_id: String,
        key_prefix: String,
    ) -> Result<Vec<kv_store::Model>, Error> {
        Ok(KVStore::find()
            .filter(kv_store::Column::NodeId.eq(node_id))
            .filter(kv_store::Column::K.starts_with(&key_prefix))
            .all(&self.connection)
            .await?)
    }

    // TODO: do not select value column or update read monitors to not separate list keys and read monitor
    pub async fn list_keys(&self, node_id: String, key_prefix: &str) -> Result<Vec<String>, Error> {
        Ok(KVStore::find()
            .filter(kv_store::Column::NodeId.eq(node_id))
            .filter(kv_store::Column::K.starts_with(key_prefix))
            .all(&self.connection)
            .await?
            .iter()
            .map(|entity| entity.k.clone())
            .collect())
    }

    pub fn list_keys_sync(&self, node_id: String, key_prefix: &str) -> Result<Vec<String>, Error> {
        tokio::task::block_in_place(move || {
            self.runtime_handle
                .block_on(async move { self.list_keys(node_id, key_prefix).await })
        })
    }

    pub fn list_values_sync(
        &self,
        node_id: String,
        key_prefix: String,
    ) -> Result<Vec<kv_store::Model>, Error> {
        tokio::task::block_in_place(move || {
            self.runtime_handle
                .block_on(async move { self.list_values(node_id, key_prefix).await })
        })
    }

    pub async fn set_value(
        &self,
        node_id: String,
        key: String,
        value: Vec<u8>,
    ) -> Result<kv_store::Model, Error> {
        let entry = match self.get_value(node_id.clone(), key.clone()).await? {
            Some(entry) => {
                let mut existing_entry: kv_store::ActiveModel = entry.into();
                existing_entry.v = ActiveValue::Set(value);
                existing_entry.update(&self.connection)
            }
            None => kv_store::ActiveModel {
                node_id: ActiveValue::Set(node_id),
                k: ActiveValue::Set(key),
                v: ActiveValue::Set(value),
                ..Default::default()
            }
            .insert(&self.connection),
        }
        .await?;

        Ok(entry)
    }

    pub fn set_value_sync(
        &self,
        node_id: String,
        key: String,
        value: Vec<u8>,
    ) -> Result<kv_store::Model, Error> {
        tokio::task::block_in_place(move || {
            self.runtime_handle
                .block_on(async move { self.set_value(node_id, key, value).await })
        })
    }

    pub async fn get_seed(&self, node_id: String) -> Result<Option<Vec<u8>>, Error> {
        self.get_value(node_id, String::from("seed"))
            .await
            .map(|model| model.map(|model| model.v))
    }

    pub async fn set_seed(&self, node_id: String, seed: Vec<u8>) -> Result<kv_store::Model, Error> {
        self.set_value(node_id, String::from("seed"), seed).await
    }

    // Note: today we assume there's only ever one macaroon for a user
    //       once there's some `bakery` functionality exposed we need to define
    //       which macaroon we return when a user unlocks their node
    pub async fn get_macaroon(&self, node_id: String) -> Result<Option<macaroon::Model>, Error> {
        Ok(Macaroon::find()
            .filter(macaroon::Column::NodeId.eq(node_id))
            .one(&self.connection)
            .await?)
    }

    pub async fn find_macaroon_by_id(
        &self,
        macaroon_id: String,
    ) -> Result<Option<macaroon::Model>, Error> {
        Ok(Macaroon::find_by_id(macaroon_id)
            .one(&self.connection)
            .await?)
    }

    pub async fn create_macaroon(
        &self,
        node_id: String,
        id: String,
        encrypted_macaroon: Vec<u8>,
    ) -> Result<macaroon::Model, Error> {
        let macaroon = macaroon::ActiveModel {
            node_id: ActiveValue::Set(node_id),
            id: ActiveValue::Set(id),
            encrypted_macaroon: ActiveValue::Set(encrypted_macaroon),
            ..Default::default()
        };

        Ok(macaroon.insert(&self.connection).await?)
    }

    pub async fn find_or_create_last_sync(
        &self,
        node_id: String,
        best_blockhash: BlockHash,
    ) -> Result<BlockHash, Error> {
        match self
            .get_value(node_id.clone(), String::from("last_blockhash_sycned"))
            .await?
        {
            Some(entry) => {
                let best_blockhash: BlockHash = deserialize(&entry.v).unwrap();
                Ok(best_blockhash)
            }
            None => {
                let serialized_blockhash = serialize(&best_blockhash);
                self.set_value(
                    node_id,
                    String::from("last_blockhash_synced"),
                    serialized_blockhash,
                )
                .await?;
                Ok(best_blockhash)
            }
        }
    }
}
