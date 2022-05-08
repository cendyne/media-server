// Copyright (C) 2022 Cendyne.
// This file is part of Cendyne Media-Server.

// Cendyne Media-Server is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.

// Cendyne Media-Server is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

table! {
    object (id) {
        id -> Integer,
        content_hash -> Text,
        content_type -> Text,
        content_encoding -> Text,
        length -> BigInt,
        file_path -> Text,
        created -> BigInt,
        modified -> BigInt,
        derived_object_id -> Nullable<Integer>,
        transforms -> Nullable<Text>,
        transforms_hash -> Nullable<Text>,
        width -> Nullable<Integer>,
        height -> Nullable<Integer>,
        content_headers -> Nullable<Text>,
        quality -> Nullable<Integer>,
    }
}

table! {
    object_blur_hash (object_id, x_components, y_components, background) {
        object_id -> Integer,
        x_components -> Integer,
        y_components -> Integer,
        background -> Text,
        hash -> Text,
    }
}

table! {
    virtual_object (id) {
        id -> Integer,
        object_path -> Text,
        default_jpeg_bg -> Nullable<Text>,
        derived_virtual_object_id -> Nullable<Integer>,
        primary_object_id -> Nullable<Integer>,
        transforms -> Nullable<Text>,
        transforms_hash -> Nullable<Text>,
    }
}

table! {
    virtual_object_relation (virtual_object_id, object_id) {
        virtual_object_id -> Integer,
        object_id -> Integer,
    }
}

joinable!(object_blur_hash -> object (object_id));
joinable!(virtual_object -> object (primary_object_id));
joinable!(virtual_object_relation -> object (object_id));
joinable!(virtual_object_relation -> virtual_object (virtual_object_id));

allow_tables_to_appear_in_same_query!(
    object,
    object_blur_hash,
    virtual_object,
    virtual_object_relation,
);
