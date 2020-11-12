
ALTER TABLE poll_submission
	ADD CONSTRAINT poll_submission_cfg_user_uniq UNIQUE (cfg, "user");
