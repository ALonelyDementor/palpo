-- Your SQL goes here
ALTER TABLE user_profiles
ADD CONSTRAINT unique_user_id UNIQUE (user_id);
