extern crate itertools; // group_by

extern crate chrono; // date and time structures and operations

extern crate hyper; // http server
extern crate futures;
extern crate url; 
extern crate percent_encoding;

#[macro_use]
extern crate rouste; // routing

#[macro_use]
extern crate serde_json; // json serialization

pub mod time_range;
pub mod monoid;
pub mod tree;
pub mod solver;
pub mod utils;
pub mod service;

use service::handle_request;
use solver::Solver;

use hyper::{ Server };
use hyper::service::service_fn;
use hyper::rt::Future;

const LOG_FILENAME: &'static str = "hn_logs.tsv";

fn main() {
    println!("Preparing data structures");
    match Solver::new(LOG_FILENAME) {
        Ok(solver) => {
            println!("Starting web server, go to http://127.0.0.1:8000");
            let server_addr = ([127, 0, 0, 1], 8000).into();
            let service = move || {
                let solver = solver.clone();
                service_fn(move |request| {
                    println!("{} {:?}", request.method(), request.uri());
                    handle_request(request, &solver)
                })
            };

            let server = Server::bind(&server_addr)
                .serve(service)
                .map_err(|error| eprintln!("Server error: {}", error));

            hyper::rt::run(server);
        },
        _ => {
            eprintln!("Failed to load data");
        }
    }
}
