use crate::database::models::ids::{DBProductId, DBProductPriceId};
use crate::models::billing::ProductMetadata;
use crate::routes::ApiError;

pub struct DBProductsTaxIdentifier {
    pub id: i32,
    pub tax_processor_id: String,
    pub product_id: DBProductId,
}

impl DBProductsTaxIdentifier {
    pub async fn get_product(
        product_id: DBProductId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Option<Self>, ApiError> {
        let maybe_row = sqlx::query!(
            "SELECT * FROM products_tax_identifiers WHERE product_id = $1",
            product_id.0,
        )
        .fetch_optional(exec)
        .await?;

        Ok(maybe_row.map(|row| DBProductsTaxIdentifier {
            id: row.id,
            tax_processor_id: row.tax_processor_id,
            product_id: DBProductId(row.product_id),
        }))
    }

    pub async fn get_price(
        price_id: DBProductPriceId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Option<Self>, ApiError> {
        let maybe_row = sqlx::query!(
            "
			SELECT pti.*
			FROM products_prices pp
			INNER JOIN products_tax_identifiers pti ON pti.product_id = pp.product_id
			WHERE pp.id = $1
			",
            price_id.0,
        )
        .fetch_optional(exec)
        .await?;

        Ok(maybe_row.map(|row| DBProductsTaxIdentifier {
            id: row.id,
            tax_processor_id: row.tax_processor_id,
            product_id: DBProductId(row.product_id),
        }))
    }
}

pub struct ProductInfo {
    pub tax_identifier: DBProductsTaxIdentifier,
    pub product_metadata: ProductMetadata,
}

pub async fn product_info_by_product_price_id(
    product_price_id: DBProductPriceId,
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
) -> Result<Option<ProductInfo>, ApiError> {
    let maybe_row = sqlx::query!(
        r#"
        SELECT
          products_tax_identifiers.*,
          products.metadata product_metadata
        FROM products_prices
        INNER JOIN products ON products.id = products_prices.product_id
        INNER JOIN products_tax_identifiers ON products_tax_identifiers.product_id = products.id
        WHERE products_prices.id = $1
        "#,
        product_price_id.0 as i64,
    )
    .fetch_optional(exec)
    .await?;

    match maybe_row {
        None => Ok(None),
        Some(row) => Ok(Some(ProductInfo {
            tax_identifier: DBProductsTaxIdentifier {
                id: row.id,
                tax_processor_id: row.tax_processor_id,
                product_id: DBProductId(row.product_id),
            },
            product_metadata: serde_json::from_value(row.product_metadata)?,
        })),
    }
}
