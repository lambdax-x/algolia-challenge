use solver::Solver;
use utils::parse::{ parse_string, parse_time_range, parse_count_param };

use hyper;
use hyper::{ Body, Request, Response, StatusCode };
use hyper::rt::Future;
use url::percent_encoding::percent_decode;
use futures::future;

use serde_json;

/// Default page content
const DEFAULT_CONTENT: &'static str = "# Algolia interview challenge

## Number of queries in a time range

Endpoint: /1/queries/count/year[-month[-day[ hour[:minutes]]]]

## K most frequent queries in a time range

Endpoint: /1/queries/popular/year[-month[-day[ hour[:minutes]]]]?size=n";

/// Wrapper for percent decoding
fn  decode_str_safe(data: &str) -> Option<String> {
    let data = data.as_bytes();
    percent_decode(data).decode_utf8()
                        .ok()
                        .map(|clean_str| clean_str.into_owned())
}

type BoxedFuture = Box<Future<Item=Response<Body>, Error=hyper::Error> + Send>;

/// Decode URI and box response for hyper
pub fn handle_request(req: Request<Body>, solver: &Solver) -> BoxedFuture {
    // Decode URI
    let uri = req.uri();
    let maybe_path = decode_str_safe(uri.path());
    let maybe_query_params = uri.query()
                                .or(Some(""))
                                .and_then(|query_str| decode_str_safe(query_str));

    if maybe_path.is_none() || maybe_query_params.is_none() {
        let mut response = Response::new(Body::empty());
        *response.status_mut() = StatusCode::BAD_REQUEST;
        return Box::new(future::ok(response));
    }

    let response = make_response(solver, &maybe_path.unwrap(), &maybe_query_params.unwrap());
    Box::new(future::ok(response))
}

/// Decode query, call solver and build response
fn make_response(solver: &Solver, path: &str, query_params: &str) -> Response<Body> {
    let mut response = Response::new(Body::empty());

    let maybe_endpoint = parse_string("/1/queries", path);
    let maybe_endpoint_count = maybe_endpoint.and_then(|(_, other)| parse_string("/count/", other)).map(|(_, other)| other);
    let maybe_endpoint_popular = maybe_endpoint.and_then(|(_, other)| parse_string("/popular/", other)).map(|(_, other)| other);

    let maybe_body = match (maybe_endpoint_count, maybe_endpoint_popular) {
        (Ok(query), _) => handle_count(solver, query),
        (_, Ok(query)) => handle_popular(solver, query, query_params),
        _ => None
    };

    if maybe_body.is_some() {
        *response.body_mut() = Body::from(maybe_body.unwrap());
    } else {
        *response.body_mut() = Body::from(DEFAULT_CONTENT);
        *response.status_mut() = StatusCode::BAD_REQUEST;
    }

    response
}

fn handle_count(solver: &Solver, query: &str) -> Option<String> {
    parse_time_range(query).map(|(from, to)| {
        json!({
            "from": from.to_string(),
            "to": to.to_string(),
            "count": solver.query_count(&from, &to)
        }).to_string()
    })
}

fn handle_popular(solver: &Solver, query: &str, query_params: &str) -> Option<String> {
    let maybe_size = query_params.split('&')
                                 .map(|param| parse_count_param(param))
                                 .filter(|maybe_size| maybe_size.is_some())
                                 .next()
                                 .and_then(|x| x);

    parse_time_range(query).and_then(|(from, to)| {
        maybe_size.map(|size| {
            let queries: serde_json::Value = solver.query_k_count(&from, &to, size)
                                                   .iter()
                                                   .map(|&(ref query, ref count)| json!({
                                                       "query": query,
                                                       "count": count
                                                   }))
                                                   .collect();
            json!({
                "from": from.to_string(),
                "to": to.to_string(),
                "queries": queries
            }).to_string()
        })
    })
}
