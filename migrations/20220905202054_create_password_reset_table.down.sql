-- Add down migration script here

ALTER TABLE
    `password_resets` DROP FOREIGN KEY `fk_password_resets_user_id`;

DROP TABLE IF EXISTS `password_resets`;