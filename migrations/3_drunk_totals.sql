ALTER TABLE drunk
ADD COLUMN score GENERATED ALWAYS AS (beer + (wine * 2) + (shots * 2) + (cocktails * 2) + (derby * 5));

ALTER TABLE drunk ADD COLUMN last_drink TEXT;
ALTER TABLE drunk ADD COLUMN last_spill DATETIME;