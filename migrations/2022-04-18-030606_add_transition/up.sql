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
    `derived_object_id` integer,
    `transforms` text,
    `transforms_hash` text,
    `width` integer,
    `height` integer,
    `content_headers` text,
    foreign key (`derived_object_id`) references `object`(`id`)
);
CREATE INDEX `object_content_hash3` on `object`(`content_hash`);
CREATE INDEX `object_file_path3` on `object`(`file_path`);
CREATE INDEX `object_transforms_hash` on `object`(`transforms_hash`);

insert into `object` select `id`, `content_hash`, `content_type`, `content_encoding`, `length`, `file_path`, `created`, `modified`, null, null, null, `width`, `height`, `content_headers` from `old_object` order by `id`;

-- Now Virtual Object
ALTER TABLE `virtual_object` RENAME TO `old_virtual_object`;
create table `virtual_object` (
    `id` integer primary key autoincrement not null,
    `object_path` text not null,
    `default_jpeg_bg` text,
    `derived_virtual_object_id` integer,
    `primary_object_id` integer,
    `transforms` text,
    `transforms_hash` text,
    foreign key (`derived_virtual_object_id`) references `virtual_object`(`id`),
    foreign key (`primary_object_id`) references `object`(`id`),
    unique(`object_path`)
);

CREATE INDEX `virtual_object_transforms_hash` on `virtual_object`(`transforms_hash`);

insert into `virtual_object` select `id`, `object_path`, null, null, null, null, null from `old_virtual_object` order by `id`;

-- Now relationship because its foreign keys need to point to the right tables

ALTER TABLE `virtual_object_relation` RENAME TO `old_virtual_object_relation`;

CREATE TABLE `virtual_object_relation` (
    `virtual_object_id` integer not null,
    `object_id` integer not null,
    primary key(`virtual_object_id`, `object_id`),
    foreign key (`virtual_object_id`) references `virtual_object`(`id`),
    foreign key (`object_id`) references `object`(`id`)
);
CREATE INDEX `virtual_object_relation_object3` on `virtual_object_relation` (`object_id`);

insert into `virtual_object_relation` select * from `old_virtual_object_relation`;
