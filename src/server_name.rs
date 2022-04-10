use rocket::{Request, Response};
use rocket::http::Header;
use rocket::fairing::{Fairing, Info, Kind};

pub struct ServerName(&'static str);

impl ServerName {
    pub fn new(name: &'static str) -> Self {
        Self(name)
    }
}

#[rocket::async_trait]
impl Fairing for ServerName {
    fn info(&self) -> Info {
        Info {
            name: "Replaces server header",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Server", self.0));
    }
}