ALTER TABLE items ADD COLUMN display_name TEXT;
INSERT INTO schema_version VALUES (2, strftime('%s', 'now'));
