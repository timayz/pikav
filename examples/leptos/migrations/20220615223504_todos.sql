-- Add migration script here
CREATE TABLE IF NOT EXISTS `todos`
(
    `id`            INTEGER PRIMARY KEY NOT NULL,
    `user_id`       TEXT                NOT NULL,
    `text`          TEXT                NOT NULL,
    `done`          BOOLEAN             NOT NULL DEFAULT 0
);
