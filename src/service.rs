use solver::Solver;
use time_range::TimeRange;

use hyper;
use hyper::{ Body, Request, Response, StatusCode };
use hyper::header::{ HeaderValue, CONTENT_TYPE };
use hyper::rt::Future;
use futures::future;

use serde_json;

use rouste::utils::*;

type BoxedFuture = Box<Future<Item=Response<Body>, Error=hyper::Error> + Send>;

type ContentType = &'static str;
const CONTENT_TYPE_TEXT: ContentType = "text/plain";
const CONTENT_TYPE_JSON: ContentType = "application/json";

/// Decode URI and box response for hyper
pub fn handle_request(req: Request<Body>, solver: &Solver) -> BoxedFuture {
    // Bind handlers with the solver
    let binded_handle_count = |version: u32, time_range: TimeRange, distinct: Option<()>| {
        handle_count(solver, version, time_range, distinct)
    };

    let binded_handle_popular = |version: u32, time_range: TimeRange, size: Option<usize>| {
        handle_popular(solver, version, time_range, size)
    };

    let uri = format!("{}?{}", req.uri().path(), req.uri().query().unwrap_or_default());

    let router = route_with![ route!(/ => handle_default)
                            , route!(/(version: u32)/queries/count/(time_range: TimeRange)?distinct => binded_handle_count)
                            , route!(/(version: u32)/queries/popular/(time_range: TimeRange)?(size: usize) => binded_handle_popular)
                            ];

    let mut response = Response::new(Body::empty());

    match router(&uri) {
        Some((content_type, content, status)) => {
            response.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static(content_type));
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

## Types

- u32: 32 bits unsigned integer
- TimeRange: YYYY[-MM[-DD[ hh[:mm]]]]

## Number of queries in a time range

Endpoint: /<version: u32>/queries/count/<time range: TimeRange>[?[distinct]]

## K most frequent queries in a time range

Endpoint: /<version: u32>/queries/popular/<time range: TimeRange>[?[size=<u32>]]";

fn handle_default() -> (ContentType, String, StatusCode) {
    (CONTENT_TYPE_TEXT, DEFAULT_CONTENT.to_string(), StatusCode::OK)
}

fn handle_count(solver: &Solver, _version: u32, time_range: TimeRange, distinct: Option<()>) -> (ContentType, String, StatusCode) {
    let count = match distinct {
        Some(_) => solver.query_distinct_count(&time_range.from, &time_range.to),
        _ => solver.query_count(&time_range.from, &time_range.to)
    };
    let body = json!({
        "from": time_range.from.to_string(),
        "to": time_range.to.to_string(),
        "count": count
    }).to_string();
    (CONTENT_TYPE_JSON, body, StatusCode::OK)
}

fn handle_popular(solver: &Solver, _version: u32, time_range: TimeRange, size: Option<usize>) -> (ContentType, String, StatusCode) {
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
    (CONTENT_TYPE_JSON, body, StatusCode::OK)
}
