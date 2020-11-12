-- poll_user

CREATE SEQUENCE poll_user_id_seq;

CREATE TABLE poll_user (
	id BIGINT NOT NULL DEFAULT nextval('poll_user_id_seq'),
	firstname TEXT NOT NULL,
	lastname TEXT,
	user_token TEXT NOT NULL,
	-- indices
	PRIMARY KEY(id),
	UNIQUE(user_token)
);

ALTER SEQUENCE poll_user_id_seq OWNED BY poll_user.id;

-- tg_user

CREATE TABLE tg_user (
	user_id TEXT NOT NULL,
	username TEXT,
	poll_user BIGINT NOT NULL,
	-- indices
	PRIMARY KEY(user_id),
	FOREIGN KEY (poll_user) REFERENCES poll_user(id)
);
