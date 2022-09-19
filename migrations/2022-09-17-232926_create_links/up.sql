-- Your SQL goes here
CREATE TABLE IF NOT EXISTS `links` (
  `link_id` varchar(12) not null primary key,
  `target` text not null,
  `control_key` varchar(255) not null,
  `added_at` timestamp not null default current_timestamp,
  `visit_count` integer not null default 0
) ENGINE=InnoDB;
