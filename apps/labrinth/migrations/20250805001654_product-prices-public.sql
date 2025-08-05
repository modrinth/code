-- Add migration script here

ALTER TABLE
  products_prices
ADD COLUMN
  public BOOLEAN NOT NULL DEFAULT true;
