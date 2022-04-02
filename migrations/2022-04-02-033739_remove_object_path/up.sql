ALTER TABLE `object` RENAME TO `old_object`;
CREATE TABLE `object` (
    `id` integer primary key autoincrement not null,
    `content_hash` text not null,
    `content_type` text not null,
    `content_encoding` text not null default('identity'),
    `length` BIGINT not null,
    `file_path` text not null,
    `created` BIGINT not null,
    `modified` BIGINT not null,
    `width` integer,
    `height` integer,
    `content_headers` text
);
CREATE INDEX `object_content_hash2` on `object`(`content_hash`);
CREATE INDEX `object_file_path2` on `object`(`file_path`);

insert into `object` select `id`, `content_hash`, `content_type`, `content_encoding`, `length`, `file_path`, `created`, `modified`, `width`, `height`, `content_headers` from `old_object` order by `id`;

ALTER TABLE `virtual_object_relation` RENAME TO `old_virtual_object_relation`;

CREATE TABLE `virtual_object_relation` (
    `virtual_object_id` integer not null,
    `object_id` integer not null,
    primary key(`virtual_object_id`, `object_id`),
    foreign key (`virtual_object_id`) references `virtual_object`(`id`),
    foreign key (`object_id`) references `object`(`id`)
);
CREATE INDEX `virtual_object_relation_object2` on `virtual_object_relation` (`object_id`);

insert into `virtual_object_relation` select * from `old_virtual_object_relation`;
