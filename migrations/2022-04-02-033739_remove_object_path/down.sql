DROP TABLE `object`
ALTER TABLE `old_object` RENAME TO `object`;
DROP TABLE `virtual_object_relation`;
ALTER TABLE `old_virtual_object_relation` RENAME TO `virtual_object_relation`;
