use crate::database::models::{
    product_item, DatabaseError, ProductId, ProductPriceId,
};
use crate::database::redis::RedisPool;
use crate::models::billing::{Price, ProductMetadata};
use dashmap::DashMap;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::convert::TryInto;

const PRODUCTS_NAMESPACE: &str = "products";

pub struct ProductItem {
    pub id: ProductId,
    pub metadata: ProductMetadata,
    pub unitary: bool,
}

struct ProductResult {
    id: i64,
    metadata: serde_json::Value,
    unitary: bool,
}

macro_rules! select_products_with_predicate {
    ($predicate:tt, $param:ident) => {
        sqlx::query_as!(
            ProductResult,
            r#"
            SELECT id, metadata, unitary
            FROM products
            "#
                + $predicate,
            $param
        )
    };
}

impl TryFrom<ProductResult> for ProductItem {
    type Error = serde_json::Error;

    fn try_from(r: ProductResult) -> Result<Self, Self::Error> {
        Ok(ProductItem {
            id: ProductId(r.id),
            metadata: serde_json::from_value(r.metadata)?,
            unitary: r.unitary,
        })
    }
}

impl ProductItem {
    pub async fn get(
        id: ProductId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Option<ProductItem>, DatabaseError> {
        Ok(Self::get_many(&[id], exec).await?.into_iter().next())
    }

    pub async fn get_many(
        ids: &[ProductId],
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<ProductItem>, DatabaseError> {
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
    ) -> Result<Vec<ProductItem>, DatabaseError> {
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
pub struct QueryProduct {
    pub id: ProductId,
    pub metadata: ProductMetadata,
    pub unitary: bool,
    pub prices: Vec<ProductPriceItem>,
}

impl QueryProduct {
    pub async fn list<'a, E>(
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<QueryProduct>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        let mut redis = redis.connect().await?;

        let res: Option<Vec<QueryProduct>> = redis
            .get_deserialized_from_json(PRODUCTS_NAMESPACE, "all")
            .await?;

        if let Some(res) = res {
            return Ok(res);
        }

        let all_products = product_item::ProductItem::get_all(exec).await?;
        let prices = product_item::ProductPriceItem::get_all_products_prices(
            &all_products.iter().map(|x| x.id).collect::<Vec<_>>(),
            exec,
        )
        .await?;

        let products = all_products
            .into_iter()
            .map(|x| QueryProduct {
                id: x.id,
                metadata: x.metadata,
                prices: prices
                    .remove(&x.id)
                    .map(|x| x.1)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|x| ProductPriceItem {
                        id: x.id,
                        product_id: x.product_id,
                        prices: x.prices,
                        currency_code: x.currency_code,
                    })
                    .collect(),
                unitary: x.unitary,
            })
            .collect::<Vec<_>>();

        redis
            .set_serialized_to_json(PRODUCTS_NAMESPACE, "all", &products, None)
            .await?;

        Ok(products)
    }
}

#[derive(Deserialize, Serialize)]
pub struct ProductPriceItem {
    pub id: ProductPriceId,
    pub product_id: ProductId,
    pub prices: Price,
    pub currency_code: String,
}

struct ProductPriceResult {
    id: i64,
    product_id: i64,
    prices: serde_json::Value,
    currency_code: String,
}

macro_rules! select_prices_with_predicate {
    ($predicate:tt, $param:ident) => {
        sqlx::query_as!(
            ProductPriceResult,
            r#"
            SELECT id, product_id, prices, currency_code
            FROM products_prices
            "#
                + $predicate,
            $param
        )
    };
}

impl TryFrom<ProductPriceResult> for ProductPriceItem {
    type Error = serde_json::Error;

    fn try_from(r: ProductPriceResult) -> Result<Self, Self::Error> {
        Ok(ProductPriceItem {
            id: ProductPriceId(r.id),
            product_id: ProductId(r.product_id),
            prices: serde_json::from_value(r.prices)?,
            currency_code: r.currency_code,
        })
    }
}

impl ProductPriceItem {
    pub async fn get(
        id: ProductPriceId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Option<ProductPriceItem>, DatabaseError> {
        Ok(Self::get_many(&[id], exec).await?.into_iter().next())
    }

    pub async fn get_many(
        ids: &[ProductPriceId],
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<ProductPriceItem>, DatabaseError> {
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
        product_id: ProductId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<ProductPriceItem>, DatabaseError> {
        let res = Self::get_all_products_prices(&[product_id], exec).await?;

        Ok(res.remove(&product_id).map(|x| x.1).unwrap_or_default())
    }

    pub async fn get_all_products_prices(
        product_ids: &[ProductId],
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<DashMap<ProductId, Vec<ProductPriceItem>>, DatabaseError> {
        let ids = product_ids.iter().map(|id| id.0).collect_vec();
        let ids_ref: &[i64] = &ids;

        use futures_util::TryStreamExt;
        let prices = select_prices_with_predicate!(
            "WHERE product_id = ANY($1::bigint[])",
            ids_ref
        )
        .fetch(exec)
        .try_fold(
            DashMap::new(),
            |acc: DashMap<ProductId, Vec<ProductPriceItem>>, x| {
                if let Ok(item) = <ProductPriceResult as TryInto<
                    ProductPriceItem,
                >>::try_into(x)
                {
                    acc.entry(item.product_id).or_default().push(item);
                }

                async move { Ok(acc) }
            },
        )
        .await?;

        Ok(prices)
    }
}
