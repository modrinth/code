use crate::database::models::{
    DBProductId, DBProductPriceId, DatabaseError, product_item,
};
use crate::database::redis::RedisPool;
use crate::models::billing::{Price, ProductMetadata};
use dashmap::DashMap;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::convert::TryInto;

const PRODUCTS_NAMESPACE: &str = "products";

pub struct DBProduct {
    pub id: DBProductId,
    pub metadata: ProductMetadata,
    pub unitary: bool,
}

struct ProductQueryResult {
    id: i64,
    metadata: serde_json::Value,
    unitary: bool,
}

macro_rules! select_products_with_predicate {
    ($predicate:tt, $param:ident) => {
        sqlx::query_as!(
            ProductQueryResult,
            r#"
            SELECT id, metadata, unitary
            FROM products
            "#
                + $predicate,
            $param
        )
    };
}

impl TryFrom<ProductQueryResult> for DBProduct {
    type Error = serde_json::Error;

    fn try_from(r: ProductQueryResult) -> Result<Self, Self::Error> {
        Ok(DBProduct {
            id: DBProductId(r.id),
            metadata: serde_json::from_value(r.metadata)?,
            unitary: r.unitary,
        })
    }
}

impl DBProduct {
    pub async fn get(
        id: DBProductId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Option<DBProduct>, DatabaseError> {
        Ok(Self::get_many(&[id], exec).await?.into_iter().next())
    }

    pub async fn get_by_type<'a, E>(
        exec: E,
        r#type: &str,
    ) -> Result<Vec<Self>, DatabaseError>
    where
        E: sqlx::PgExecutor<'a>,
    {
        let maybe_row = select_products_with_predicate!(
            "WHERE metadata ->> 'type' = $1",
            r#type
        )
        .fetch_all(exec)
        .await?;

        maybe_row
            .into_iter()
            .map(|r| r.try_into().map_err(Into::into))
            .collect()
    }

    pub async fn get_many(
        ids: &[DBProductId],
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<DBProduct>, DatabaseError> {
        let ids = ids.iter().map(|id| id.0).collect_vec();
        let ids_ref: &[i64] = &ids;
        let results = select_products_with_predicate!(
            "WHERE id = ANY($1::bigint[])",
            ids_ref
        )
        .fetch_all(exec)
        .await?;

        Ok(results
            .into_iter()
            .map(|r| r.try_into())
            .collect::<Result<Vec<_>, serde_json::Error>>()?)
    }

    pub async fn get_all(
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<DBProduct>, DatabaseError> {
        let one = 1;
        let results = select_products_with_predicate!("WHERE 1 = $1", one)
            .fetch_all(exec)
            .await?;

        Ok(results
            .into_iter()
            .map(|r| r.try_into())
            .collect::<Result<Vec<_>, serde_json::Error>>()?)
    }
}

#[derive(Deserialize, Serialize)]
pub struct QueryProductWithPrices {
    pub id: DBProductId,
    pub metadata: ProductMetadata,
    pub unitary: bool,
    pub prices: Vec<DBProductPrice>,
}

impl QueryProductWithPrices {
    /// Lists products with at least one public price.
    pub async fn list_purchaseable<'a, E>(
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<Self>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        let mut redis = redis.connect().await?;

        let res: Option<Vec<QueryProductWithPrices>> = redis
            .get_deserialized_from_json(PRODUCTS_NAMESPACE, "all")
            .await?;

        if let Some(res) = res {
            return Ok(res);
        }

        let all_products = product_item::DBProduct::get_all(exec).await?;
        let prices =
            product_item::DBProductPrice::get_all_public_products_prices(
                &all_products.iter().map(|x| x.id).collect::<Vec<_>>(),
                exec,
            )
            .await?;

        let products = all_products
            .into_iter()
            .filter_map(|x| {
                Some(QueryProductWithPrices {
                    id: x.id,
                    metadata: x.metadata,
                    prices: prices
                        .remove(&x.id)
                        .map(|x| x.1)?
                        .into_iter()
                        .map(|x| DBProductPrice {
                            id: x.id,
                            product_id: x.product_id,
                            prices: x.prices,
                            currency_code: x.currency_code,
                        })
                        .collect(),
                    unitary: x.unitary,
                })
            })
            .collect::<Vec<_>>();

        redis
            .set_serialized_to_json(PRODUCTS_NAMESPACE, "all", &products, None)
            .await?;

        Ok(products)
    }

    pub async fn list_by_product_type<'a, E>(
        exec: E,
        r#type: &str,
    ) -> Result<Vec<Self>, DatabaseError>
    where
        E: sqlx::PgExecutor<'a> + Copy,
    {
        let all_products = DBProduct::get_by_type(exec, r#type).await?;
        let prices = DBProductPrice::get_all_products_prices(
            &all_products.iter().map(|x| x.id).collect::<Vec<_>>(),
            exec,
        )
        .await?;

        let products = all_products
            .into_iter()
            .filter_map(|x| {
                Some(QueryProductWithPrices {
                    id: x.id,
                    metadata: x.metadata,
                    prices: prices
                        .remove(&x.id)
                        .map(|x| x.1)?
                        .into_iter()
                        .map(|x| DBProductPrice {
                            id: x.id,
                            product_id: x.product_id,
                            prices: x.prices,
                            currency_code: x.currency_code,
                        })
                        .collect(),
                    unitary: x.unitary,
                })
            })
            .collect::<Vec<_>>();

        Ok(products)
    }
}

#[derive(Deserialize, Serialize)]
pub struct DBProductPrice {
    pub id: DBProductPriceId,
    pub product_id: DBProductId,
    pub prices: Price,
    pub currency_code: String,
}

struct ProductPriceQueryResult {
    id: i64,
    product_id: i64,
    prices: serde_json::Value,
    currency_code: String,
}

macro_rules! select_prices_with_predicate {
    ($predicate:tt, $param1:ident) => {
        select_prices_with_predicate!($predicate, $param1, )
    };

    ($predicate:tt, $($param:ident,)+) => {
        sqlx::query_as!(
            ProductPriceQueryResult,
            r#"
            SELECT id, product_id, prices, currency_code
            FROM products_prices
            "#
                + $predicate,
            $($param),+
        )
    };
}

impl TryFrom<ProductPriceQueryResult> for DBProductPrice {
    type Error = serde_json::Error;

    fn try_from(r: ProductPriceQueryResult) -> Result<Self, Self::Error> {
        Ok(DBProductPrice {
            id: DBProductPriceId(r.id),
            product_id: DBProductId(r.product_id),
            prices: serde_json::from_value(r.prices)?,
            currency_code: r.currency_code,
        })
    }
}

impl DBProductPrice {
    pub async fn get(
        id: DBProductPriceId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Option<DBProductPrice>, DatabaseError> {
        Ok(Self::get_many(&[id], exec).await?.into_iter().next())
    }

    pub async fn get_many(
        ids: &[DBProductPriceId],
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<DBProductPrice>, DatabaseError> {
        let ids = ids.iter().map(|id| id.0).collect_vec();
        let ids_ref: &[i64] = &ids;
        let results = select_prices_with_predicate!(
            "WHERE id = ANY($1::bigint[])",
            ids_ref
        )
        .fetch_all(exec)
        .await?;

        Ok(results
            .into_iter()
            .map(|r| r.try_into())
            .collect::<Result<Vec<_>, serde_json::Error>>()?)
    }

    pub async fn get_all_product_prices(
        product_id: DBProductId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<DBProductPrice>, DatabaseError> {
        let res = Self::get_all_products_prices(&[product_id], exec).await?;

        Ok(res.remove(&product_id).map(|x| x.1).unwrap_or_default())
    }

    pub async fn get_all_public_product_prices(
        product_id: DBProductId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<DBProductPrice>, DatabaseError> {
        let res =
            Self::get_all_public_products_prices(&[product_id], exec).await?;

        Ok(res.remove(&product_id).map(|x| x.1).unwrap_or_default())
    }

    /// Gets all public prices for the given products. If a product has no public price,
    /// it won't be included in the resulting map.
    pub async fn get_all_public_products_prices(
        product_ids: &[DBProductId],
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<DashMap<DBProductId, Vec<DBProductPrice>>, DatabaseError> {
        Self::get_all_products_prices_with_visibility(
            product_ids,
            Some(true),
            exec,
        )
        .await
    }

    pub async fn get_all_products_prices(
        product_ids: &[DBProductId],
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<DashMap<DBProductId, Vec<DBProductPrice>>, DatabaseError> {
        Self::get_all_products_prices_with_visibility(product_ids, None, exec)
            .await
    }

    async fn get_all_products_prices_with_visibility(
        product_ids: &[DBProductId],
        public_filter: Option<bool>,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<DashMap<DBProductId, Vec<DBProductPrice>>, DatabaseError> {
        let ids = product_ids.iter().map(|id| id.0).collect_vec();
        let ids_ref: &[i64] = &ids;

        use futures_util::TryStreamExt;

        let predicate = |acc: DashMap<DBProductId, Vec<DBProductPrice>>, x| {
            if let Ok(item) = <ProductPriceQueryResult as TryInto<
                DBProductPrice,
            >>::try_into(x)
            {
                acc.entry(item.product_id).or_default().push(item);
            }

            async move { Ok(acc) }
        };

        let prices = match public_filter {
            None => {
                select_prices_with_predicate!(
                    "WHERE product_id = ANY($1::bigint[])",
                    ids_ref,
                )
                .fetch(exec)
                .try_fold(DashMap::new(), predicate)
                .await?
            }

            Some(public) => {
                select_prices_with_predicate!(
                    "WHERE product_id = ANY($1::bigint[]) AND public = $2",
                    ids_ref,
                    public,
                )
                .fetch(exec)
                .try_fold(DashMap::new(), predicate)
                .await?
            }
        };

        Ok(prices)
    }
}
