create table `object` (
    `id` integer primary key autoincrement not null,
    `content_hash` text not null,
    `content_type` text not null,
    `content_encoding` text not null default('identity'),
    `length` integer not null,
    `object_path` text not null,
    `file_path` text not null,
    `created` integer not null,
    `modified` integer not null,
    `width` integer,
    `height` integer,
    `content_headers` text,
    unique(`object_path`)
);
create table `virtual_object` (
    `id` integer primary key autoincrement not null,
    `object_path` text not null,
    unique(`object_path`)
);
create table `virtual_object_relation` (
    `virtual_object_id` integer not null,
    `object_id` integer not null,
    primary key(`virtual_object_id`, `object_id`)
);
create index `virtual_object_relation_object` on `virtual_object_relation` (`object_id`);