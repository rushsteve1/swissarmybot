ALTER TABLE drunk ADD COLUMN water INTEGER NOT NULL DEFAULT 0;

ALTER TABLE drunk DROP COLUMN score;
ALTER TABLE drunk
ADD COLUMN score GENERATED ALWAYS AS (beer + (wine * 2) + (shots * 2) + (cocktails * 2) + (derby * 3));