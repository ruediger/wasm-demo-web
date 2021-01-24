use warp::Filter;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!("hello" / String)
        .map(|name| warp::reply::html(format!("<em>Hello, {}!</em>", name)));

    let index = warp::get().and(warp::path::end()).and(warp::fs::file("../../webdata/index.html"));

    let webdata_files = warp::get().and(warp::path("webdata")).and(warp::fs::dir("../../webdata/"));
    let pkg_files = warp::get().and(warp::path("pkg")).and(warp::fs::dir("../../pkg/"));

    let routes = hello.or(webdata_files.or(pkg_files.or(index)));

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
