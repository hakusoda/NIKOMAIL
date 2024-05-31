CREATE TABLE servers (
	id bigint primary key,
	forum_channel_id bigint,
	created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE topics (
	id bigint primary key,
	author_id bigint NOT NULL,
	server_id bigint NOT NULL,
	created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP
);