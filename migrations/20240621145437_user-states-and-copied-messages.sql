CREATE TABLE user_states (
	id bigint primary key,
	current_topic_id bigint
);

CREATE TABLE relayed_messages (
	id bigserial primary key,
	author_id bigint NOT NULL,
	topic_id bigint NOT NULL REFERENCES topics (id) ON DELETE CASCADE,
	source_channel_id bigint NOT NULL,
	source_message_id bigint NOT NULL,
	relayed_channel_id bigint NOT NULL,
	relayed_message_id bigint NOT NULL,
	is_topic_starter boolean NOT NULL DEFAULT false,
	created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP
);