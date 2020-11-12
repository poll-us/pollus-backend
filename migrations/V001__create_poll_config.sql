-- poll_config

CREATE SEQUENCE poll_config_id_seq;

CREATE TABLE poll_config (
	id BIGINT NOT NULL DEFAULT nextval('poll_config_id_seq'),
	poll TEXT NOT NULL,
	date DATE NOT NULL,
	-- indices
	PRIMARY KEY (id),
	UNIQUE (poll, date)
);

ALTER SEQUENCE poll_config_id_seq OWNED BY poll_config.id;

-- poll_config_time

CREATE TABLE poll_config_time (
	cfg BIGINT NOT NULL,
	time TIME NOT NULL,
	-- indices
	PRIMARY KEY (cfg, time),
	FOREIGN KEY (cfg) REFERENCES poll_config(id)
);
