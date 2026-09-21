use crate::database::models::{DBProductId, DBProductPriceId, product_item};
use crate::models::billing::{Price, ProductMetadata};
use dashmap::DashMap;
use eyre::{Result, WrapErr};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::convert::TryInto;
use xredis::RedisPool;

const PRODUCTS_NAMESPACE: &str = "products:v4";

pub struct DBProduct {
    pub id: DBProductId,
    pub metadata: ProductMetadata,
    pub unitary: bool,
    pub name: Option<String>,
}

struct ProductQueryResult {
    id: i64,
    metadata: serde_json::Value,
    unitary: bool,
    name: Option<String>,
}

macro_rules! select_products_with_predicate {
    ($predicate:tt, $param:expr) => {
        sqlx::query_as!(
            ProductQueryResult,
            r#"
            SELECT products.id, products.metadata, products.unitary, products.name
            FROM products
            "#
                + $predicate,
            $param
        )
    };
}

impl TryFrom<ProductQueryResult> for DBProduct {
    type Error = serde_json::Error;

    fn try_from(
        r: ProductQueryResult,
    ) -> std::result::Result<Self, Self::Error> {
        Ok(DBProduct {
            id: DBProductId(r.id),
            metadata: serde_json::from_value(r.metadata)?,
            unitary: r.unitary,
            name: r.name,
        })
    }
}

impl DBProduct {
    pub async fn get(
        id: DBProductId,
        exec: impl crate::database::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Option<DBProduct>> {
        Ok(Self::get_many(&[id], exec)
            .await
            .wrap_err("fetching product")?
            .into_iter()
            .next())
    }

    pub async fn get_price(
        id: DBProductPriceId,
        exec: impl crate::database::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Option<DBProduct>> {
        let maybe_row = select_products_with_predicate!(
            "INNER JOIN products_prices pp ON pp.id = $1
			WHERE products.id = pp.product_id",
            id.0
        )
        .fetch_optional(exec)
        .await
        .wrap_err("fetching product by price")?;

        maybe_row
            .map(TryInto::try_into)
            .transpose()
            .wrap_err("deserializing product metadata")
    }

    pub async fn get_by_type<'a, E>(exec: E, r#type: &str) -> Result<Vec<Self>>
    where
        E: sqlx::PgExecutor<'a>,
    {
        let maybe_row = select_products_with_predicate!(
            "WHERE metadata ->> 'type' = $1",
            r#type
        )
        .fetch_all(exec)
        .await
        .wrap_err("fetching products by type")?;

        maybe_row
            .into_iter()
            .map(TryInto::try_into)
            .collect::<std::result::Result<Vec<_>, _>>()
            .wrap_err("deserializing product metadata")
    }

    pub async fn get_many(
        ids: &[DBProductId],
        exec: impl crate::database::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<DBProduct>> {
        let ids = ids.iter().map(|id| id.0).collect_vec();
        let ids_ref: &[i64] = &ids;
        let results = select_products_with_predicate!(
            "WHERE id = ANY($1::bigint[])",
            ids_ref
        )
        .fetch_all(exec)
        .await
        .wrap_err("fetching products")?;

        results
            .into_iter()
            .map(TryInto::try_into)
            .collect::<std::result::Result<Vec<_>, _>>()
            .wrap_err("deserializing product metadata")
    }

    pub async fn get_all(
        exec: impl crate::database::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<DBProduct>> {
        let one = 1;
        let results = select_products_with_predicate!("WHERE 1 = $1", one)
            .fetch_all(exec)
            .await
            .wrap_err("fetching all products")?;

        results
            .into_iter()
            .map(TryInto::try_into)
            .collect::<std::result::Result<Vec<_>, _>>()
            .wrap_err("deserializing product metadata")
    }
}

#[derive(Deserialize, Serialize)]
pub struct QueryProductWithPrices {
    pub id: DBProductId,
    pub metadata: ProductMetadata,
    pub unitary: bool,
    #[serde(default)]
    pub name: Option<String>,
    pub prices: Vec<DBProductPrice>,
}

impl QueryProductWithPrices {
    /// Lists products with at least one public price.
    pub async fn list_purchaseable<'a, E>(
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<Self>>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        {
            let mut redis = redis
                .connect()
                .await
                .wrap_err("connecting to Redis for purchasable products")?;
            let key = redis.key().metadata(PRODUCTS_NAMESPACE, "all");

            let res: Option<Vec<QueryProductWithPrices>> = redis
                .get_deserialized(&key)
                .await
                .wrap_err("fetching purchasable products from cache")?;

            if let Some(res) = res {
                return Ok(res);
            }
        }

        let all_products = product_item::DBProduct::get_all(exec)
            .await
            .wrap_err("fetching all products")?;
        let prices =
            product_item::DBProductPrice::get_all_public_products_prices(
                &all_products.iter().map(|x| x.id).collect::<Vec<_>>(),
                exec,
            )
            .await
            .wrap_err("fetching public product prices")?;

        let products = all_products
            .into_iter()
            .filter_map(|x| {
                Some(QueryProductWithPrices {
                    id: x.id,
                    metadata: x.metadata,
                    name: x.name,
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

        let mut redis = redis
            .connect()
            .await
            .wrap_err("connecting to Redis to cache purchasable products")?;
        let key = redis.key().metadata(PRODUCTS_NAMESPACE, "all");

        redis
            .set_serialized(&key, &products, None)
            .await
            .wrap_err("caching purchasable products")?;

        Ok(products)
    }

    pub async fn list_by_product_type<'a, E>(
        exec: E,
        r#type: &str,
    ) -> Result<Vec<Self>>
    where
        E: sqlx::PgExecutor<'a> + Copy,
    {
        let all_products = DBProduct::get_by_type(exec, r#type)
            .await
            .wrap_err("fetching products by type")?;
        let prices = DBProductPrice::get_all_products_prices(
            &all_products.iter().map(|x| x.id).collect::<Vec<_>>(),
            exec,
        )
        .await
        .wrap_err("fetching product prices")?;

        let products = all_products
            .into_iter()
            .filter_map(|x| {
                Some(QueryProductWithPrices {
                    id: x.id,
                    metadata: x.metadata,
                    name: x.name,
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

    fn try_from(
        r: ProductPriceQueryResult,
    ) -> std::result::Result<Self, Self::Error> {
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
        exec: impl crate::database::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Option<DBProductPrice>> {
        Ok(Self::get_many(&[id], exec)
            .await
            .wrap_err("fetching product price")?
            .into_iter()
            .next())
    }

    pub async fn get_many(
        ids: &[DBProductPriceId],
        exec: impl crate::database::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<DBProductPrice>> {
        let ids = ids.iter().map(|id| id.0).collect_vec();
        let ids_ref: &[i64] = &ids;
        let results = select_prices_with_predicate!(
            "WHERE id = ANY($1::bigint[])",
            ids_ref
        )
        .fetch_all(exec)
        .await
        .wrap_err("fetching product prices")?;

        results
            .into_iter()
            .map(TryInto::try_into)
            .collect::<std::result::Result<Vec<_>, _>>()
            .wrap_err("deserializing product prices")
    }

    pub async fn get_all_product_prices(
        product_id: DBProductId,
        exec: impl crate::database::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<DBProductPrice>> {
        let res = Self::get_all_products_prices(&[product_id], exec)
            .await
            .wrap_err("fetching product prices")?;

        Ok(res.remove(&product_id).map(|x| x.1).unwrap_or_default())
    }

    pub async fn get_all_public_product_prices(
        product_id: DBProductId,
        exec: impl crate::database::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<DBProductPrice>> {
        let res = Self::get_all_public_products_prices(&[product_id], exec)
            .await
            .wrap_err("fetching public product prices")?;

        Ok(res.remove(&product_id).map(|x| x.1).unwrap_or_default())
    }

    /// Gets all public prices for the given products. If a product has no public price,
    /// it won't be included in the resulting map.
    pub async fn get_all_public_products_prices(
        product_ids: &[DBProductId],
        exec: impl crate::database::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<DashMap<DBProductId, Vec<DBProductPrice>>> {
        Self::get_all_products_prices_with_visibility(
            product_ids,
            Some(true),
            exec,
        )
        .await
        .wrap_err("fetching public product prices")
    }

    pub async fn get_all_products_prices(
        product_ids: &[DBProductId],
        exec: impl crate::database::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<DashMap<DBProductId, Vec<DBProductPrice>>> {
        Self::get_all_products_prices_with_visibility(product_ids, None, exec)
            .await
            .wrap_err("fetching product prices")
    }

    async fn get_all_products_prices_with_visibility(
        product_ids: &[DBProductId],
        public_filter: Option<bool>,
        exec: impl crate::database::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<DashMap<DBProductId, Vec<DBProductPrice>>> {
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

            async move { Ok::<_, sqlx::Error>(acc) }
        };

        let prices = match public_filter {
            None => select_prices_with_predicate!(
                "WHERE product_id = ANY($1::bigint[])",
                ids_ref,
            )
            .fetch(exec)
            .try_fold(DashMap::new(), predicate)
            .await
            .wrap_err("fetching product prices")?,

            Some(public) => select_prices_with_predicate!(
                "WHERE product_id = ANY($1::bigint[]) AND public = $2",
                ids_ref,
                public,
            )
            .fetch(exec)
            .try_fold(DashMap::new(), predicate)
            .await
            .wrap_err("fetching public product prices")?,
        };

        Ok(prices)
    }
}
