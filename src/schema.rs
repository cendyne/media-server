table! {
    object (id) {
        id -> Integer,
        content_hash -> Text,
        content_type -> Text,
        content_encoding -> Text,
        length -> Integer,
        object_path -> Text,
        file_path -> Text,
        created -> Integer,
        modified -> Integer,
        width -> Nullable<Integer>,
        height -> Nullable<Integer>,
        content_headers -> Nullable<Text>,
    }
}

table! {
    virtual_object (id) {
        id -> Integer,
        object_path -> Text,
    }
}

table! {
    virtual_object_relation (virtual_object_id, object_id) {
        virtual_object_id -> Integer,
        object_id -> Integer,
    }
}

allow_tables_to_appear_in_same_query!(object, virtual_object, virtual_object_relation,);
