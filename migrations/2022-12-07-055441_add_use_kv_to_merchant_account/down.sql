-- This file should undo anything in `up.sql`

ALTER TABLE merchant_account DROP COLUMN storage_scheme;

DROP TYPE "MerchantStorageScheme";
