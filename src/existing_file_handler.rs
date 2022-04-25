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

use crate::image_operations::*;
use crate::models::Object;
use crate::sqlite::Pool;
use crate::ByteContent;
use crate::ContentEncodingValue;
use crate::FileContent;
use crate::{parse_existing_file_request, search_existing_file_query};
use rocket::http::{Method, Status};
use rocket::route::{Handler, Outcome, Route};
use rocket::State;
use rocket::{Data, Request};

#[derive(Clone)]
pub struct ExistingFileHandler();

#[rocket::async_trait]
impl Handler for ExistingFileHandler {
    async fn handle<'r>(&self, req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r> {
        let sem = match req.guard::<&State<ImageSemaphore>>().await {
            rocket::outcome::Outcome::Success(sem) => sem,
            rocket::outcome::Outcome::Forward(_) => return Outcome::forward(data),
            rocket::outcome::Outcome::Failure(_) => {
                return Outcome::failure(Status::InternalServerError)
            }
        };
        // TODO shorten some how?
        let pool = match req.guard::<&State<Pool>>().await {
            rocket::outcome::Outcome::Success(pool) => pool,
            rocket::outcome::Outcome::Forward(_) => return Outcome::forward(data),
            rocket::outcome::Outcome::Failure(_) => {
                return Outcome::failure(Status::InternalServerError)
            }
        };
        // TODO shorten some how?
        let conn = if let Ok(conn) = pool.get() {
            conn
        } else {
            return Outcome::failure(Status::InternalServerError);
        };

        let query = parse_existing_file_request(req);

        println!("Transformations? {:?}", query.transformations());

        let query_transformations = query.transformations();

        // Search for virtual object first
        let object: Object = if let Ok(Some(object)) = search_existing_file_query(&conn, query) {
            object
        } else {
            return Outcome::forward(data);
        };

        match query_transformations {
            Some(transformations) => {
                let quality = req.query_value::<u8>("q").transpose().unwrap_or(None);
                let image_type = req
                    .query_value::<ImageFormat>("ty")
                    .transpose()
                    .unwrap_or(None);

                let encoded_image = match read_transform_encode(
                    &object.file_path,
                    transformations,
                    quality,
                    image_type,
                    sem,
                )
                .await
                {
                    Ok(encoded_image) => encoded_image,
                    Err(err) => {
                        println!("Could not encode image {}", err);
                        return Outcome::failure(Status::InternalServerError);
                    }
                };
                let content_type = match encoded_image.format.content_type() {
                    Ok(content_type) => content_type,
                    Err(err) => {
                        println!("Unknown content type {}", err);
                        return Outcome::failure(Status::InternalServerError);
                    }
                };
                let content = ByteContent::from_bytes(
                    encoded_image.bytes,
                    content_type,
                    ContentEncodingValue::Identity,
                    None,
                );

                Outcome::from(req, content)
            }
            None => {
                let file = match FileContent::load(object).await {
                    Ok(file) => file,
                    Err(err) => {
                        println!("File content expected but could not load: {}", err);
                        return Outcome::failure(Status::InternalServerError);
                    }
                };

                Outcome::from(req, file)
            }
        }
    }
}

impl From<ExistingFileHandler> for Vec<Route> {
    fn from(handler: ExistingFileHandler) -> Vec<Route> {
        vec![Route::new(Method::Get, "/<..>", handler)]
    }
}
