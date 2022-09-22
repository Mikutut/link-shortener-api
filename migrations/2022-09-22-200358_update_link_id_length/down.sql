-- This file should undo anything in `up.sql`
ALTER TABLE `links` MODIFY COLUMN `link_id` VARCHAR(12) NOT NULL;