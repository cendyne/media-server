use crate::sqlite::Pool;
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

        // Search for virtual object first
        let object = if let Ok(Some(object)) = search_existing_file_query(&conn, query) {
            object
        } else {
            return Outcome::forward(data);
        };
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

impl From<ExistingFileHandler> for Vec<Route> {
    fn from(handler: ExistingFileHandler) -> Vec<Route> {
        vec![Route::new(Method::Get, "/<..>", handler)]
    }
}