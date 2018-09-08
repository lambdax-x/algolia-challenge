use solver::Solver;
use time_range::TimeRange;

use hyper;
use hyper::{ Body, Request, Response, StatusCode };
use hyper::rt::Future;
use futures::future;

use serde_json;

use rouste::utils::*;

type BoxedFuture = Box<Future<Item=Response<Body>, Error=hyper::Error> + Send>;

/// Decode URI and box response for hyper
pub fn handle_request(req: Request<Body>, solver: &Solver) -> BoxedFuture {
    // Bind handlers with the solver
    let binded_handle_count = |version: u32, time_range: TimeRange| {
        handle_count(solver, version, time_range)
    };

    let binded_handle_popular = |version: u32, time_range: TimeRange, size: Option<usize>| {
        handle_popular(solver, version, time_range, size)
    };

    let uri = format!("{}?{}", req.uri().path(), req.uri().query().unwrap_or_default());

    let router = route_with![ route!(/ => handle_default)
                            , route!(/(version: u32)/queries/count/(time_range: TimeRange) => binded_handle_count)
                            , route!(/(version: u32)/queries/popular/(time_range: TimeRange)?(size: usize) => binded_handle_popular)
                            ];

    let mut response = Response::new(Body::empty());

    match router(&uri) {
        Some((content, status)) => {
            *response.body_mut() = Body::from(content);
            *response.status_mut() = status;
        },

        None => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    }
    Box::new(future::ok(response))
}

const DEFAULT_CONTENT: &'static str = "# Algolia interview challenge

## Number of queries in a time range

Endpoint: /1/queries/count/year[-month[-day[ hour[:minutes]]]]

## K most frequent queries in a time range

Endpoint: /1/queries/popular/year[-month[-day[ hour[:minutes]]]]?size=n";

fn handle_default() -> (String, StatusCode) {
    (DEFAULT_CONTENT.to_string(), StatusCode::OK)
}

fn handle_count(solver: &Solver, _version: u32, time_range: TimeRange) -> (String, StatusCode) {
    let body = json!({
        "from": time_range.from.to_string(),
        "to": time_range.to.to_string(),
        "count": solver.query_count(&time_range.from, &time_range.to)
    }).to_string();
    (body, StatusCode::OK)
}

fn handle_popular(solver: &Solver, _version: u32, time_range: TimeRange, size: Option<usize>) -> (String, StatusCode) {
    const DEFAULT_SIZE: usize = 10;
    let k_queries = solver.query_k_count(&time_range.from, &time_range.to, size.unwrap_or(DEFAULT_SIZE));
    let k_queries_json: serde_json::Value = k_queries.iter()
                                                     .map(|&(ref query, ref count)| json!({
                                                         "query": query,
                                                         "count": count
                                                     })).collect();
    let body = json!({
        "from": time_range.from.to_string(),
        "to": time_range.to.to_string(),
        "queries": k_queries_json
    }).to_string();
    (body, StatusCode::OK)
}
