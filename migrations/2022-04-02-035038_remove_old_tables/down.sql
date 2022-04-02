CREATE TABLE `old_object` (
    `id` integer primary key autoincrement not null,
    `content_hash` text not null,
    `content_type` text not null,
    `content_encoding` text not null default('identity'),
    `length` BIGINT not null,
    `object_path` text not null,
    `file_path` text not null,
    `created` BIGINT not null,
    `modified` BIGINT not null,
    `width` integer,
    `height` integer,
    `content_headers` text,
    unique(`object_path`)
);
CREATE INDEX `object_content_hash` on `old_object`(`content_hash`);
CREATE INDEX `object_file_path` on `old_object`(`file_path`);


insert into `old_object` select `id`, `content_hash`, `content_type`, `content_encoding`, `length`, `file_path` as `object_path`, `file_path`, `created`, `modified`, `width`, `height`, `content_headers` from `object` order by `id`;


CREATE TABLE `old_virtual_object_relation` (
    `virtual_object_id` integer not null,
    `object_id` integer not null,
    primary key(`virtual_object_id`, `object_id`),
    foreign key (`virtual_object_id`) references `virtual_object`(`id`),
    foreign key (`object_id`) references `object`(`id`)
);
CREATE INDEX `virtual_object_relation_object` on `old_virtual_object_relation` (`object_id`);
insert into `old_virtual_object_relation` select * from `virtual_object_relation`;
