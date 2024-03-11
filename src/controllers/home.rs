use loco_rs::prelude::*;

use crate::views::home::HomeResponse;

async fn current() -> Result<Json<HomeResponse>> {
    format::json(HomeResponse::new("loco"))
}

async fn currents() -> Result<Json<HomeResponse>> {
    format::json(HomeResponse::new("loco-nice"))
}

pub fn routes() -> Routes {
    Routes::new()
        .add("/", get(current))
        .add("/nice", get(currents))
}
