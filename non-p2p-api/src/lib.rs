mod models;
use models::{create_node, select_nodes};
use worker::*;

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();

    router
        .post_async(
            "/nodes",
            |req, ctx| async move { create_node(req, ctx).await },
        )
        .get_async("/nodes", |_req, ctx| async move { select_nodes(ctx).await })
        .get_async("/", |_req, _ctx| async move {
            Response::error("Not found", 404)
        })
        .run(req, env)
        .await
}
