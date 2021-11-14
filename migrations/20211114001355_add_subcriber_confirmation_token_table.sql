-- Add migration script here
CREATE TABLE subscriber_confirmation_token(
	confirmation_token uuid NOT NULL,
	PRIMARY KEY (confirmation_token),
	subscriber uuid NOT NULL
		REFERENCES subscriptions (id)
)
