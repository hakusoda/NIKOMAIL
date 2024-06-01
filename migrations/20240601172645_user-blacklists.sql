ALTER TABLE servers
ADD blacklisted_user_ids bigint[] NOT NULL DEFAULT '{}'::bigint[];