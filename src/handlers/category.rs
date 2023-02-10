use askama::Template;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use std::{collections::HashMap, sync::Arc};

use crate::{
    errors::AppError, models::dtos::meta::Meta, services::get_categories_metas_sorted_by_name,
    utilities::templates::HtmlTemplate, AppState,
};

#[derive(Template)]
#[template(path = "index.html.j2")]
struct CategoryTemplate {
    metas: Vec<Meta>,
}

pub async fn get_categories_handler(
    State(state): State<Arc<AppState>>,
    Path(params): Path<HashMap<String, String>>,
) -> Result<impl IntoResponse, AppError> {
    let category_name = params.get("category");

    let redis_con = state.databases.redis.new_connection().await?;

    match category_name {
        Some(category_name) => {
            let metas = get_categories_metas_sorted_by_name(redis_con, category_name).await?;

            let template = CategoryTemplate { metas };

            Ok(HtmlTemplate(template))
        }
        None => Err(AppError::Custom(String::from("missing parameter"))),
    }
}
