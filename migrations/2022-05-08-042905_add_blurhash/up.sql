CREATE TABLE `object_blur_hash` (
    `object_id` integer not null,
    `x_components` integer not null,
    `y_components` integer not null,
    `background` text not null,
    `hash` text not null,
    primary key(`object_id`, `x_components`, `y_components`, `background`),
    foreign key (`object_id`) references `object`(`id`)
);
CREATE INDEX `object_blur_hash_components` on `object_blur_hash`(`object_id`, `x_components`, `y_components`);

