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

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use std::collections::HashSet;

use crate::models::{
    NewVirtualObject, Object, ReplaceVirtualObjectRelation, UpdateTransformedVirtualObject,
    VirtualObject,
};
use crate::sqlite::last_insert_rowid;

pub fn find_virtual_object_by_object_path(
    conn: &SqliteConnection,
    path: &str,
) -> Result<Option<VirtualObject>, String> {
    use crate::schema::virtual_object::dsl::*;
    let result = virtual_object
        .filter(object_path.eq(path))
        .first(conn)
        .optional()
        .map_err(|err| format!("{}", err))?;
    Ok(result)
}

pub fn find_virtual_object_by_object_paths(
    conn: &SqliteConnection,
    paths: &[&str],
) -> Result<Option<VirtualObject>, String> {
    use crate::schema::virtual_object::dsl::*;
    let result = virtual_object
        .filter(object_path.eq_any(paths))
        .first(conn)
        .optional()
        .map_err(|err| format!("{}", err))?;
    Ok(result)
}

struct VirtualObjectError;

impl diesel::result::DatabaseErrorInformation for VirtualObjectError {
    fn message(&self) -> &str {
        "Virtual Object may have already been inserted, retry"
    }
    fn details(&self) -> Option<&str> {
        None
    }
    fn hint(&self) -> Option<&str> {
        None
    }
    fn table_name(&self) -> Option<&str> {
        Some("virtual_object")
    }
    fn column_name(&self) -> Option<&str> {
        None
    }
    fn constraint_name(&self) -> Option<&str> {
        None
    }
}

pub fn find_or_create_virtual_object_by_object_path(
    conn: &SqliteConnection,
    path: &str,
) -> Result<VirtualObject, String> {
    match find_virtual_object_by_object_path(conn, path)? {
        Some(virtual_object) => Ok(virtual_object),
        None => {
            use crate::schema::virtual_object;
            // cannot use get_result on Sqlite
            // Hint.. newer sqlite has returning..
            // feature returning_clauses_for_sqlite_3_35 has not been released yet
            conn.transaction::<VirtualObject, diesel::result::Error, _>(|| {
                let result = diesel::insert_into(virtual_object::table)
                    .values(NewVirtualObject {
                        object_path: path.to_string(),
                        default_jpeg_bg: None,
                        derived_virtual_object_id: None,
                        primary_object_id: None,
                        transforms: None,
                        transforms_hash: None,
                    })
                    .execute(conn)?;
                if result > 0 {
                    use crate::schema::virtual_object::dsl::*;
                    let last_id = diesel::select(last_insert_rowid)
                        .get_result::<i32>(conn)
                        .map_err(|_| diesel::result::Error::RollbackTransaction)?;
                    let record = virtual_object
                        .filter(id.eq(last_id))
                        .first(conn)
                        .map_err(|_| diesel::result::Error::RollbackTransaction)?;
                    println!("Created new virtual object {}", last_id);
                    Ok(record)
                } else {
                    let err = VirtualObjectError;
                    Err(diesel::result::Error::DatabaseError(
                        diesel::result::DatabaseErrorKind::UniqueViolation,
                        Box::new(err),
                    ))
                }
            })
            .map_err(|e| format!("{}", e))
        }
    }
}

pub fn find_related_objects_to_virtual_object(
    conn: &SqliteConnection,
    virtual_object: &VirtualObject,
) -> Result<Vec<Object>, String> {
    use crate::schema::virtual_object_relation::dsl::*;
    let result = virtual_object_relation
        .inner_join(crate::schema::virtual_object::table)
        .inner_join(crate::schema::object::table)
        .filter(virtual_object_id.eq(&virtual_object.id))
        .select(crate::schema::object::all_columns)
        .load(conn)
        .map_err(|err| format!("{}", err))?;
    Ok(result)
}

/* TODO use around insertion
conn.transaction::<_, diesel::result::Error, _>(|| {
    delete(opts, &conn);
    Ok(())
})
.unwrap()
*/

pub fn remove_virtual_object_relations(
    conn: &SqliteConnection,
    objects: &[&Object],
    virtual_object: &VirtualObject,
) -> Result<(), String> {
    use crate::schema::virtual_object_relation::dsl::*;
    if objects.is_empty() {
        return Ok(());
    }
    let ids = objects.iter().map(|o| o.id);
    println!("Removing objects {:?}", ids);
    let targets = virtual_object_relation
        .filter(object_id.eq_any(ids))
        .filter(virtual_object_id.eq(&virtual_object.id));
    diesel::delete(targets)
        .execute(conn)
        .map_err(|err| format!("{}", err))?;
    Ok(())
}

pub fn add_virtual_object_relations(
    conn: &SqliteConnection,
    objects: &[Object],
    virtual_object_ref: &VirtualObject,
) -> Result<(), String> {
    if objects.is_empty() {
        return Ok(());
    }
    // Have to annotate it so that the DSL doesn't create some
    // crazy recursion type checking exception
    // Ibzan recommends explicit type annotation on collect() use
    let relations: Vec<ReplaceVirtualObjectRelation> = objects
        .iter()
        .map(|o| ReplaceVirtualObjectRelation {
            virtual_object_id: virtual_object_ref.id,
            object_id: o.id,
        })
        .collect();
    diesel::replace_into(crate::schema::virtual_object_relation::table)
        .values(relations)
        .execute(conn)
        .map_err(|err| format!("{}", err))?;
    Ok(())
}

pub fn replace_virtual_object_relations(
    conn: &SqliteConnection,
    objects: &[Object],
    virtual_object: &VirtualObject,
) -> Result<(), String> {
    // This method could be a lot more optimal,
    // but due to how infrequent it is used, this remains to be optimized
    let mut to_have = HashSet::new();
    for object in objects {
        to_have.insert(object.id);
    }
    let has = find_related_objects_to_virtual_object(conn, virtual_object)?;
    let has_ids: HashSet<i32> = has.iter().map(|o| o.id).collect();
    // println!("Has: {:?}", has);
    let to_keep_ids: HashSet<i32> = has_ids.intersection(&to_have).copied().collect();
    // println!("To Keep Ids: {:?}", to_keep_ids);
    let to_remove: Vec<&Object> = has
        .iter()
        .filter(|o| !to_keep_ids.contains(&o.id))
        .collect();
    // println!("To Remove: {:?}", to_remove);
    remove_virtual_object_relations(conn, &to_remove, virtual_object)?;
    // Add does a replace into, no need to do another difference
    add_virtual_object_relations(conn, objects, virtual_object)?;
    Ok(())
}

pub fn update_transformed_virtual_object(
    conn: &SqliteConnection,
    id: i32,
    update: UpdateTransformedVirtualObject,
) -> Result<(), String> {
    use crate::schema::virtual_object;
    let count = diesel::update(virtual_object::table)
        .set(&update)
        .filter(virtual_object::id.eq(&id))
        .execute(conn)
        .map_err(|err| format!("{}", err))?;
    println!("Updated {}", count);
    Ok(())
}
