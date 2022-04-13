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

use crate::virtual_object::{
    find_related_objects_to_virtual_object, find_virtual_object_by_object_paths,
};
use diesel::sqlite::SqliteConnection;

use crate::content_encoding::ContentEncodingValue;
use crate::models::Object;
use crate::parsing::grab_basename;
use crate::transformations::TransformationList;

// use rocket::http::ContentType;
use rocket::request::Request;

pub fn find_object_by_parameters(
    conn: &SqliteConnection,
    paths: &[&str],
    width: Option<i32>,
    height: Option<i32>,
    content_type: Option<&str>,
    content_encoding: Option<ContentEncodingValue>,
) -> Result<Option<Object>, String> {
    println!("Looking for virtual object by path {:?}", paths);
    println!(
        "With type {:?} and encoding {:?}",
        content_type, content_encoding
    );
    // TODO supply extension so it can try the path with and without the extension
    let virtual_object = match find_virtual_object_by_object_paths(conn, paths) {
        Ok(Some(virtual_object)) => virtual_object,
        Ok(None) => {
            println!("Could not find virtual object");
            return Ok(None);
        }
        Err(_) => {
            return Ok(None);
        }
    };
    println!("Found virtual object {:?}", virtual_object);
    // TODO find only related objects that match content type
    // TODO consider content encoding
    let objects = find_related_objects_to_virtual_object(conn, &virtual_object)?;
    // println!("Found objects {:?}", objects);
    if objects.is_empty() {
        println!("Bailing out early, objects is empty");
        return Ok(None);
    }
    let same_extension: Vec<Object> = objects
        .into_iter()
        .filter(|o| match &content_type {
            None => true,
            Some(v) => o.content_type == *v,
        })
        .filter(|o| match &content_encoding {
            None => true,
            Some(v) => ContentEncodingValue::from_database(&o.content_encoding) == *v,
        })
        .collect();
    // Bail out early
    if same_extension.is_empty() {
        println!("No matching extension");
        return Ok(None);
    }
    // TODO
    println!("Looking for closest {:?}, {:?}", width, height);
    let closest = same_extension.iter().reduce(|left, right| {
        println!(
            "Folding left:{:?}, right:{:?}",
            (left.id, left.width, left.height),
            (right.id, right.width, right.height)
        );
        match (
            left.width,
            left.height,
            right.width,
            right.height,
            width,
            height,
        ) {
            // ---------EXACT-MATCHES--------------------
            // Keep left if exact match
            (Some(wl), Some(hl), _, _, Some(w), Some(h)) if wl == w && hl == h => left,
            // Keep right if exact match
            (_, _, Some(wr), Some(hr), Some(w), Some(h)) if wr == w && hr == h => right,
            // Keep left if width matches exactly and height is smaller than width
            (Some(wl), _, _, _, Some(w), Some(h)) if wl == w && h <= w => left,
            // Keep left if height matches exactly and width is smaller than height
            (_, Some(hl), _, _, Some(w), Some(h)) if hl == h && w <= h => left,
            // Keep right if width matches exactly and height is smaller than width
            (_, _, Some(wr), _, Some(w), Some(h)) if wr == w && h <= w => right,
            // Keep right if height matches exactly and width is smaller than height
            (_, _, _, Some(hr), Some(w), Some(h)) if hr == h && w <= h => right,
            // Keep right if width matches exactly
            (_, _, Some(wr), _, Some(w), None) if wr == w => right,
            // Keep right if height matches exactly
            (_, _, _, Some(hr), None, Some(h)) if hr == h => right,

            // ------------------------------------------
            // Bias right if smaller than left but greater than desired width
            (Some(wl), _, Some(wr), _, Some(w), Some(h))
                if wr >= w && (wr < wl || wl < w) && h <= w =>
            {
                right
            }
            (Some(wl), _, Some(wr), _, Some(w), None) if wr >= w && (wr < wl || wl < w) => right,
            // Bias right if smaller than left but greater than desired height
            (_, Some(hl), _, Some(hr), Some(w), Some(h))
                if hr >= h && (hr < hl || hl < h) && w <= h =>
            {
                right
            }
            (_, Some(hl), _, Some(hr), None, Some(h)) if hr >= h && (hr < hl || hl < h) => right,
            // Bias right if width is a greater size
            (None, _, Some(wr), _, Some(w), Some(h)) if wr >= w && h <= w => right,
            (None, _, Some(wr), _, Some(w), None) if wr >= w => right,
            // Bias right if height is a greater size
            (_, None, _, Some(hr), Some(w), Some(h)) if hr >= h && w <= h => right,
            (_, None, _, Some(hr), None, Some(h)) if hr >= h => right,
            // Keep left if right is not greater or equal to
            _ => left,
        }
    });
    println!("Found closest {:?}", closest);
    Ok(closest.cloned())
}

pub struct ExistingFileRequestQuery {
    raw_path: String,
    path_ranges: Vec<std::ops::Range<usize>>,
    width: Option<i32>,
    height: Option<i32>,
    content_type: Option<String>,
    content_encoding: Option<ContentEncodingValue>,
    transformations: Option<TransformationList>,
}

impl ExistingFileRequestQuery {
    pub fn transformations(&self) -> Option<TransformationList> {
        self.transformations.clone()
    }
}

pub fn parse_existing_file_request(req: &Request<'_>) -> ExistingFileRequestQuery {
    // r for resize
    // TODO detect if requested path begins with r<width>x<height>/
    // TODO extract extension
    // TODO extract encoding (identity, br, gzip, etc.)
    let raw_path = req.routed_segments(0..).collect::<Vec<_>>().join("/");
    // TODO or use path supplied width & height
    let mut width = req.query_value::<i32>("w").transpose().unwrap_or(None);
    let mut height = req.query_value::<i32>("h").transpose().unwrap_or(None);
    let mut first_segment_is_dimensions = false;
    let first_segment = req.routed_segment(0);
    // let first_length = first_segment.map(|s| s.len()).unwrap_or(0);
    match first_segment {
        None => {}
        Some(segment) => {
            if let Some(segment_slice) = segment.strip_prefix('r') {
                match segment_slice.find('x') {
                    None => {
                        // Technically r100 is fine (width 100)
                        println!("Found r'{}'", segment_slice);
                        if let Ok(w) = segment_slice.parse::<i32>() {
                            width = Some(w);
                            first_segment_is_dimensions = true;
                            println!("Width updated to {}", w);
                        }
                    }
                    Some(x_index) => {
                        let width_slice = &segment_slice[..x_index];
                        let height_slice = &segment_slice[x_index + 1..];
                        println!("Found r'{}'x'{}'", width_slice, height_slice);
                        // Technically rx100 is fine too (height 100)
                        if !width_slice.is_empty() {
                            if let Ok(w) = width_slice.parse::<i32>() {
                                width = Some(w);
                                first_segment_is_dimensions = true;
                                println!("Width updated to {}", w);
                            }
                        }
                        if !height_slice.is_empty() {
                            if let Ok(h) = height_slice.parse::<i32>() {
                                height = Some(h);
                                first_segment_is_dimensions = true;
                                println!("Height updated to {}", h);
                            }
                        }
                    }
                }
            }
        }
    }
    // TODO don't use path, piece it out so .tar.gz => tar.gz is the extension
    // and that the content_type is tar and the content_encoding is gzip

    let mut skip_first = 0..raw_path.len();
    let mut include_full = true;
    if first_segment_is_dimensions {
        match raw_path.find('/') {
            None => {}
            Some(slash_index) => {
                skip_first = slash_index + 1..raw_path.len();
                let slice = &raw_path[slash_index + 1..raw_path.len()];
                println!("Without path params: {}", slice);
                include_full = false;
            }
        }
    }

    let parsed_path = grab_basename(&raw_path);

    let first_extension = parsed_path
        .content_type_ext_range
        .clone()
        .map(|r| skip_first.start..r.start - 1);
    let second_extension = parsed_path
        .content_encoding_ext_range
        .clone()
        .map(|r| skip_first.start..r.start - 1);
    let content_type = parsed_path
        .find_content_type()
        .map(|(top, sub)| format!("{}/{}", top, sub));
    let content_encoding = parsed_path.find_content_encoding();

    // println!("Encoding: {:?}, Extension: {:?}", parsed_path.content_encoding_ext_range.map(|r| &raw_path[r]), parsed_path.content_type_ext_range.map(|r| &raw_path[r]));
    // TODO convert to content type combo and encoding

    let mut path_ranges = Vec::with_capacity(3);
    if skip_first.start > 0 {
        path_ranges.push(skip_first);
    }
    if let Some(range) = first_extension {
        path_ranges.push(range);
    }
    if let Some(range) = second_extension {
        path_ranges.push(range);
    }

    // Raw path is added last
    if include_full {
        path_ranges.push(0..raw_path.len());
    }

    let transformations = req
        .query_value::<TransformationList>("t")
        .transpose()
        .unwrap_or(None);

    ExistingFileRequestQuery {
        raw_path,
        path_ranges,
        width,
        height,
        content_type,
        content_encoding,
        transformations,
    }
}

pub fn search_existing_file_query(
    conn: &SqliteConnection,
    query: ExistingFileRequestQuery,
) -> Result<Option<Object>, String> {
    let paths: Vec<&str> = query
        .path_ranges
        .iter()
        .map(|range| &query.raw_path[range.clone()])
        .collect();
    let content_type = query.content_type;
    find_object_by_parameters(
        conn,
        &paths,
        query.width,
        query.height,
        content_type.as_deref(),
        query.content_encoding,
    )
}
