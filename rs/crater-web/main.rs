extern crate iron;
extern crate hyper;
extern crate router;
extern crate mount;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate crater_db;
extern crate rustc_serialize;
extern crate crater_engine;

use std::error::Error as StdError;
use std::fmt::{Display, Formatter};
use std::fmt::Error as FmtError;
use std::convert::From;
use std::io::Error as IoError;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use iron::prelude::*;
use iron::status;
use iron::mime::Mime;
use router::Router;
use mount::Mount;
use rustc_serialize::json;

fn main() {
    run().unwrap();
}

fn run() -> Result<(), Error> {
    try!(env_logger::init());

    let config = try!(load_config());

    // Start the job engine that listens to the pulse server, creates
    // taskcluster tasks, and updates the database with results.
    try!(start_engine(config.engine));

    // Blocks until the process is killed
    run_web_server(config.db)
}

fn start_engine(engine_config: crater_engine::Config) -> Result<(), Error> {
    Ok(())
}

fn run_web_server(db_config: crater_db::Config) -> Result<(), Error> {
    let static_router = static_router();
    let api_router = api_router(db_config);

    let mut mount = Mount::new();
    mount.mount("/api/v1/", api_router);
    mount.mount("/", static_router);

    let _ = try!(Iron::new(mount).http("localhost:3000"));

    return Ok(());
}

#[derive(RustcEncodable, RustcDecodable)]
struct Config {
    db: crater_db::Config,
    engine: crater_engine::Config
}

fn load_config() -> Result<Config, Error> {
    let mut path = try!(::std::env::current_dir());
    path.push("crater-web-config.json");

    let mut file = try!(File::open(path));

    let mut s = String::new();
    try!(file.read_to_string(&mut s));

    return Ok(try!(json::decode(&s)));
}

fn static_router() -> Router {
    let mut router = Router::new();
    router.get("/", move |_: &mut Request| {
        let (payload, mime_type) = try!(get_static_file_and_mime_type("index.html"));
        Ok(Response::with((status::Ok, payload)).set(mime_type))
    });
    router.get("*", move |r: &mut Request| {
        let last = r.url.path.last().expect("path is supposed to be non-empty");
        let filename = if last == "" {
            String::from("index.html")
        } else {
            last.clone()
        };

        let (payload, mime_type) = try!(get_static_file_and_mime_type(&filename));
        
        Ok(Response::with((status::Ok, payload)).set(mime_type))
    });

    return router;
}

fn get_static_file_and_mime_type(name: &str) -> Result<(String, Mime), Error> {
    let payload = try!(get_static_file(&name));
    let mime_type = known_mime_type(try!(get_mime_type(&name)));

    return Ok((payload, mime_type));
}

fn known_mime_type(mime_type: &str) -> Mime {
    mime_type.parse().ok().expect("shouldn't create mime types that don't parse")
}

/// Loads a file from the './static' directory
fn get_static_file(name: &str) -> Result<String, Error> {
    let mut path = try!(::std::env::current_dir());

    let asset_dir = "static";
    
    path.push(asset_dir);
    path.push(name);

    let mut file = try!(File::open(path));

    let mut s = String::new();
    try!(file.read_to_string(&mut s));

    return Ok(s);
}

fn get_mime_type(name: &str) -> Result<&'static str, Error> {
    if name.ends_with(".html") {
        Ok("text/html")
    } else if name.ends_with(".js") {
        Ok("application/x-javascript")
    } else if name.ends_with(".css") {
        Ok("text/css")
    } else {
        Err(Error::BadMimeType)
    }
}

#[derive(Debug)]
pub enum Error {
    FileNotFound,
    BadMimeType,
    JsonError,
    HyperError,
    LoggerError
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::FileNotFound => "file not found",
            Error::BadMimeType => "bad mime type",
            Error::JsonError => "json error",
            Error::HyperError => "hyper error",
            Error::LoggerError => "logger error"
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        f.write_str(self.description())
    }
}

impl From<Error> for IronError {
    fn from(e: Error) -> IronError {
        IronError::new(e, status::NotFound)
    }
}

impl From<IoError> for Error {
    fn from(_: IoError) -> Error {
        Error::FileNotFound
    }
}

impl From<json::DecoderError> for Error {
    fn from(_: json::DecoderError) -> Error {
        Error::JsonError
    }
}

impl From<hyper::Error> for Error {
    fn from(_: hyper::Error) -> Error {
        Error::HyperError
    }
}

impl From<log::SetLoggerError> for Error {
    fn from(_: log::SetLoggerError) -> Error {
        Error::LoggerError
    }
}

fn api_router(db_config: crater_db::Config) -> Router {
    let api_ctxt_master = Arc::new(api::Ctxt::new(db_config));
    let mut router = Router::new();

    let api_ctxt = api_ctxt_master.clone();
    router.get("/toolchain_build_results/:toolchain", move |r: &mut Request| {
        let router = r.extensions.get::<Router>().unwrap();
        let toolchain = router.find("toolchain").unwrap(); // FIXME unwrap
        let payload = try!(api_ctxt.toolchain_build_results(&toolchain));
        Ok(Response::with((status::Ok, payload)).set(known_mime_type("application/json")))
    });

    return router;
}

mod api {
    use super::Error;
    use crater_db;

    pub struct Ctxt {
        db_config: crater_db::Config
    }

    impl Ctxt {
        pub fn new(db_config: crater_db::Config) -> Ctxt {
            Ctxt {
                db_config: db_config
            }
        }

        pub fn toolchain_build_results(&self, toolchain: &str) -> Result<String, Error> {
            unimplemented!()
        }
    }
}
