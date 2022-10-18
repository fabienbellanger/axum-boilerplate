-- Add up migration script here

ALTER TABLE `users`
ADD
    COLUMN `rate_limit` INT NOT NULL DEFAULT 30 AFTER `roles`;