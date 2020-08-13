#![deny(warnings)]

use shared_types::{ProcessInfo, UpdateResp};
use serde::Serialize;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
};
use warp::filters::path::FullPath;
use warp::Filter;
use psutil::process;
use std::collections::HashMap;

/// A serialized message to report in JSON format.
#[derive(Serialize)]
struct ErrorMessage<'a> {
    code: u16,
    message: &'a str,
}

#[tokio::main]
async fn main() {

    let get_processes = warp::get().and(warp::path("processes")).and_then(|| async move {
        let processes = process::processes().unwrap(); // TODO: error handling, probably

        let mut process_map = HashMap::new();

        for process in processes.into_iter() {
            if let Ok(mut process) = process {
                if let Ok(name) = process.name() {
                    let mem_percent = process.memory_percent().unwrap(); // FIXME- error_handling
                    let cpu_percent = process.cpu_percent().unwrap();
                    let process_info = ProcessInfo {
                        pid: process.pid(),
                        mem_percent,
                        cpu_percent,
                        cmd_line: process.cmdline().unwrap().unwrap_or_else(|| "n/a".to_string()),
                    };

                    let vec = process_map.entry(name).or_insert(Vec::new());

                    vec.push(process_info);
                }
            }
        }

        let resp = UpdateResp{process_map};

        let resp: Result<_, warp::Rejection> = Ok(warp::reply::json(&resp));
        resp
    });

    // serve static content embeded in binary
    let static_route = warp::get().and(warp::path::full()).map(|path: FullPath| {
        match frontend::STATIC_LOOKUP.get(&path.as_str()) {
            None => {
                println!("lookup failed for {}", &path.as_str());
                hyper::Response::builder()
                    .status(hyper::StatusCode::NOT_FOUND)
                    .body(hyper::Body::empty())
                    .unwrap()

            }            Some(resp) => {
                println!("lookup passed for {}", &path.as_str());
                resp
            }
        }
    });

    // index_route overrides any static index files in static_route
    let routes = get_processes.or(static_route);

    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080);
    println!("running on localhost:8080");
    warp::serve(routes).run(socket).await;
}
