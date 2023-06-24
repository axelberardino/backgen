use axum::{
    extract::{Path, Query},
    response::Html,
    routing::get,
    Router,
};
use backgen::gen_image::generate_images;
use minijinja::render;
use rand::Rng;
use std::{collections::HashMap, net::SocketAddr};
use tower_http::services::ServeDir;
use tracing::Level;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /gen` goes to `gen`
        .route("/gen/:id", get(gen_path_handler))
        .route("/gen", get(gen_query_handler))
        .nest_service("/assets", ServeDir::new("assets"));

    let addr = SocketAddr::from(([127, 0, 0, 1], 5000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// basic handler that responds with a static string
async fn root() -> Html<&'static str> {
    Html(HOME_PAGE_TEMPLATE)
}

// Generate page from the query like /gen?id=42
async fn gen_query_handler(Query(params): Query<HashMap<String, String>>) -> Html<String> {
    match params.get("id") {
        Some(id) => {
            if id.is_empty() {
                gen_handler(None).await
            } else {
                match id.parse::<u64>() {
                    Ok(id) => gen_handler(Some(id)).await,
                    Err(err) => {
                        tracing::error!("Error occured {err}");
                        Html(format!("Error occured {err}"))
                    }
                }
            }
        }
        None => gen_handler(None).await,
    }
}

// Generate page from the path like /gen/42
async fn gen_path_handler(Path(id): Path<u64>) -> Html<String> {
    gen_handler(Some(id)).await
}

// Helper which genreate images, blurhash and the final html page.
async fn gen_handler(id: Option<u64>) -> Html<String> {
    let id = id.unwrap_or_else(|| {
        let mut rng = rand::thread_rng();
        rng.gen()
    });

    let root = "assets";
    let gen_dest = format!("{root}/{id}.gen.png");
    let blur_dest = format!("{root}/{id}.blur.png");
    match generate_images(Some(id), &gen_dest, &blur_dest) {
        Ok(blurhash) => {
            let r = render!(
                GEN_PAGE_TEMPLATE,
                id => id,
                blurhash => blurhash,
                gen_dest => gen_dest,
                blur_dest => blur_dest,
            );
            Html(r)
        }
        Err(err) => {
            tracing::error!("Error occured {err}");
            Html(format!("Error occured {err}"))
        }
    }
}

// Template for the home page
const HOME_PAGE_TEMPLATE: &str = r#"
<!doctype html>

<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">

  <title>Background generator</title>
  <meta name="description" content="Background generator">
  <meta name="author" content="Ten Ten">
</head>

<body>
    <h1>Generation of background</h1>
    <p>
        <form action="/gen" action="get">
            <input type="text" name="id" />
            <input type="submit" value="generate" />
        </form>
    </p>
</body>
</html>
"#;

// Template for the gen page
const GEN_PAGE_TEMPLATE: &str = r#"
<!doctype html>

<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">

  <title>Background generator</title>
  <meta name="description" content="Background generator">
  <meta name="author" content="Ten Ten">
</head>

<body>
    <h1>Generation of background for {{ id|title }}</h1>
    <p>BlurHash is: {{ blurhash }}</p>
    <p>
        <h2>Generated Image<h2>
        <img src="/{{ gen_dest }}" />
    </p>
    <p>
        <h2>Blurhash Image<h2>
        <img src="/{{ blur_dest }}" />
    </p>
</body>
</html>
"#;
