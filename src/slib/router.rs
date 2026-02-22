extern crate iron;

use std::collections::HashMap;
use iron::prelude::*;
use iron::{Handler, status, AfterMiddleware, headers};
use iron::mime::Mime;
use iron::modifier::Modifier;
use log::{debug};

pub struct Router {
    routes: HashMap<String, HashMap<String, Box<dyn Handler>>>,
}

impl Router {
    pub fn new() -> Self {
        return Router{ routes: HashMap::new() };
    }

    /// Add a route to the map for a GET request
    pub fn get<H>(&mut self, path: &str, handler: H) 
            where H: Handler {
        self.do_insert(path, "GET".to_string(), handler);
    }

    /// Add a route to the map for a PUT request
    pub fn put<H>(&mut self, path: &str, handler: H) 
            where H: Handler {
        self.do_insert(path, "PUT".to_string(), handler);
    }

    /// Add a route to the map for a POST request
    pub fn post<H>(&mut self, path: &str, handler: H) 
            where H: Handler {
        self.do_insert(path, "POST".to_string(), handler);
    }

    /// Add a route to the map for a DELETE request
    pub fn delete<H>(&mut self, path: &str, handler: H) 
            where H: Handler {
        self.do_insert(path, "DELETE".to_string(), handler);
    }

    fn do_insert<H>(&mut self, path: &str, method: String, handler: H)
            where H: Handler {
        if self.routes.contains_key(path) {
            let tmp = self.routes.get_mut(path).unwrap();
            tmp.insert(method, Box::new(handler));
        } else {
            let mut new: HashMap<String, Box<dyn Handler>> = HashMap::new();
            new.insert(method, Box::new(handler));
            self.routes.insert(path.to_string(), new);
        }       
    }
}

impl Handler for Router {
    /// This is the middleware handler for requests
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        // first we try to match an actual path
        let mut path = "/".to_string();
        path += &req.url.path().join("/");

        let mut result = match self.routes.get(&path) {
            Some(map) => {
                match map.get(&req.method.to_string()) {
                    Some(handler) => Some(handler.handle(req)),
                    None => None,
                }
            }
            None => None,
        };
        
        // check for "*" items
        if result.is_none() {
            result = match self.routes.get(&"*".to_string()) {
                Some(map) => {
                    match map.get(&req.method.to_string()) {
                        Some(handler) => Some(handler.handle(req)),
                        None => return Ok(Response::with(status::NotFound)),
                    }
                }
                None => return Ok(Response::with(status::NotFound)),
            }
        }
        debug!("Final Result: {:?}", &result);
        
        return result.unwrap();
    }
}

pub struct RunAfter {
    response: String,
}

impl RunAfter {
    pub fn new(resp: String) -> Self {
        return RunAfter{ response: resp };
    }
}

impl AfterMiddleware for RunAfter {
    fn after(&self, _req: &mut Request, mut resp: Response) 
            -> IronResult<Response> {
        let m: Mime = "text/plain; charset=utf8".parse().unwrap();
        if resp.headers.get::<headers::ContentType>() == None {
            resp.headers.set(headers::ContentType(m));
        }

        self.response.clone().modify(&mut resp);
        println!("{:?}", resp);
        println!("{}", self.response);
        println!();
        println!("----");
        println!();

        return Ok(resp);
    }
}