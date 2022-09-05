-- Add up migration script here

CREATE TABLE
    IF NOT EXISTS `password_resets` (
        `user_id` varchar(36) NOT NULL,
        `token` varchar(36) NOT NULL,
        `expired_at` datetime(3) NOT NULL,
        PRIMARY KEY (`user_id`),
        KEY `idx_password_resets_expired_at` (`expired_at`)
    ) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4;

ALTER TABLE `password_resets`
ADD
    CONSTRAINT `fk_password_resets_user_id` FOREIGN KEY (`user_id`) REFERENCES `users`(`id`);