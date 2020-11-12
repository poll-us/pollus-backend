-- poll_submission

CREATE SEQUENCE poll_submission_id_seq;

CREATE TABLE poll_submission (
	id BIGINT NOT NULL DEFAULT nextval('poll_submission_id_seq'),
	cfg BIGINT NOT NULL,
	"user" BIGINT NOT NULL,
	-- indices
	PRIMARY KEY(id),
	UNIQUE(id, cfg),
	FOREIGN KEY (cfg) REFERENCES poll_config(id),
	FOREIGN KEY ("user") REFERENCES poll_user(id)
);

ALTER SEQUENCE poll_submission_id_seq OWNED BY poll_submission.id;

-- poll_submission_time

CREATE TABLE poll_submission_time (
	submission BIGINT NOT NULL,
	cfg BIGINT NOT NULL,
	time TIME NOT NULL,
	value SMALLINT,
	-- indices
	PRIMARY KEY(submission, time),
	FOREIGN KEY (submission) REFERENCES poll_submission(id),
	FOREIGN KEY (cfg) REFERENCES poll_config(id),
	FOREIGN KEY (cfg, time) REFERENCES poll_config_time(cfg, time)
);
