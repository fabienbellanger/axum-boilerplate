-- Add up migration script here
CREATE TABLE IF NOT EXISTS `users` (
    `id` varchar(36) NOT NULL,
    `username` varchar(127) NOT NULL,
    `password` varchar(191) NOT NULL,
    `lastname` varchar(63) NOT NULL,
    `firstname` varchar(63) NOT NULL,
    `roles` VARCHAR(63),
    `created_at` datetime(3) NOT NULL,
    `updated_at` datetime(3) NOT NULL,
    `deleted_at` datetime(3) DEFAULT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `username` (`username`),
    KEY `idx_users_roles` (`roles`),
    KEY `idx_users_password` (`password`),
    KEY `idx_users_deleted_at` (`deleted_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
