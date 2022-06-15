CREATE TABLE IF NOT EXISTS `todos`
(
    `id`            INTEGER PRIMARY KEY AUTOINCREMENT,
    `user_id`       TEXT             NOT NULL,
    `text`          TEXT             NOT NULL,
    `done`   INT(1)           NOT NULL
);
