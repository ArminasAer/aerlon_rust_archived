use std::fs;

use chrono::{DateTime, Utc};
use comrak::{
    adapters::SyntaxHighlighterAdapter,
    markdown_to_html_with_plugins,
    plugins::syntect::{SyntectAdapter, SyntectAdapterBuilder},
    ComrakOptions, ComrakPlugins,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sqlx::{query_as, FromRow, Pool, Postgres};
use syntect::{
    highlighting::{Theme, ThemeSet},
    html::{ClassStyle, ClassedHTMLGenerator},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};
use uuid::Uuid;

use crate::errors::AppError;

pub struct CustomSyntectAdapter {}
impl SyntaxHighlighterAdapter for CustomSyntectAdapter {
    fn write_highlighted(
        &self,
        output: &mut dyn std::io::Write,
        lang: Option<&str>,
        code: &str,
    ) -> std::io::Result<()> {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let syntax = syntax_set.find_syntax_by_token(lang.unwrap()).unwrap();
        let mut html_generator =
            ClassedHTMLGenerator::new_with_class_style(syntax, &syntax_set, ClassStyle::Spaced);
        for line in LinesWithEndings::from(code) {
            html_generator.parse_html_for_line_which_includes_newline(line);
        }
        let output_html = html_generator.finalize();

        write!(output, "{}", output_html)
    }

    fn write_pre_tag(
        &self,
        output: &mut dyn std::io::Write,
        attributes: std::collections::HashMap<String, String>,
    ) -> std::io::Result<()> {
        output.write_all(b"<pre>")
    }

    fn write_code_tag(
        &self,
        output: &mut dyn std::io::Write,
        attributes: std::collections::HashMap<String, String>,
    ) -> std::io::Result<()> {
        write!(output, "<code class=\"{}\">", attributes["class"])
    }
}

#[derive(FromRow, Debug, Serialize, Deserialize, Clone)]
pub struct Post {
    #[serde(rename = "id")]
    pub post_id: Option<Uuid>,
    pub date: DateTime<Utc>,
    pub slug: String,
    pub title: String,
    pub series: String,
    pub categories: Vec<String>,
    pub markdown: String,
    pub post_snippet: String,
    pub series_snippet: String,
    pub published: bool,
    pub featured: bool,
    #[serde(rename = "created_at")]
    pub post_created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updated_at")]
    pub post_updated_at: Option<DateTime<Utc>>,
}

// state modifiers
impl Post {
    pub fn convert_markdown_to_html(&mut self) {
        let mut options = ComrakOptions::default();
        options.extension.autolink = true;
        options.extension.header_ids = Some(String::from(""));
        options.render.unsafe_ = true;
        let mut plugins = ComrakPlugins::default();

        let adapter = CustomSyntectAdapter {};
        plugins.render.codefence_syntax_highlighter = Some(&adapter);

        let converted = markdown_to_html_with_plugins(&self.markdown, &options, &plugins);
        self.markdown = converted;
    }
}

// redis methods
impl Post {
    // pub async fn get_posts_redis(mut redis_con: RedisConnection) -> Result<Vec<Self>, AppError> {
    //     let posts: Vec<Self> = redis_con.get_cache_redis().await?;

    //     Ok(posts)
    // }

    // pub async fn get_post_by_slug(
    //     redis_con: RedisConnection,
    //     post_slug: &str,
    // ) -> Result<Self, AppError> {
    //     let posts = Self::get_posts_redis(redis_con).await?;

    //     match posts.into_iter().find(|p| p.slug == post_slug) {
    //         Some(mut p) => {
    //             p.convert_markdown_to_html();
    //             Ok(p)
    //         }
    //         None => Err(AppError::Custom(format!("{} not found", post_slug))),
    //     }
    // }
}

// postgres site methods checking for published state
impl Post {
    pub async fn get_published_posts_postgres(
        postgres_pool: &Pool<Postgres>,
    ) -> Result<Vec<Self>, AppError> {
        let posts = query_as!(Post, r#"select id as "post_id?", date, slug, title, series, categories, markdown, post_snippet, series_snippet, published, featured, created_at as "post_created_at?", updated_at as "post_updated_at?" from post where published = true"#)
            .fetch_all(postgres_pool)
            .await?;

        Ok(posts)
    }

    #[allow(unused)]
    pub async fn get_published_post_by_id_postgres(
        postgres_pool: &Pool<Postgres>,
        post_id: &str,
    ) -> Result<Self, AppError> {
        let id = Uuid::parse_str(&post_id)?;

        let post = query_as!(Post, r#"select id as "post_id?", date, slug, title, series, categories, markdown, post_snippet, series_snippet, published, featured, created_at as "post_created_at?", updated_at as "post_updated_at?" from post where id = $1 and published = true"#, &id)
            .fetch_one(postgres_pool)
            .await?;

        Ok(post)
    }
}

// postgres admin api methods
impl Post {
    pub async fn get_posts_postgres(postgres_pool: &Pool<Postgres>) -> Result<Vec<Self>, AppError> {
        let posts = query_as!(Post, r#"select id as "post_id?", date, slug, title, series, categories, markdown, post_snippet, series_snippet, published, featured, created_at as "post_created_at?", updated_at as "post_updated_at?" from post"#)
            .fetch_all(postgres_pool)
            .await?;

        Ok(posts)
    }

    pub async fn get_post_by_id_postgres(
        postgres_pool: &Pool<Postgres>,
        post_id: &str,
    ) -> Result<Self, AppError> {
        let id = Uuid::parse_str(&post_id)?;

        let post = query_as!(Post, r#"select id as "post_id?", date, slug, title, series, categories, markdown, post_snippet, series_snippet, published, featured, created_at as "post_created_at?", updated_at as "post_updated_at?" from post where id = $1"#, &id)
            .fetch_one(postgres_pool)
            .await?;

        Ok(post)
    }

    #[allow(unused)]
    pub async fn create_post_postgres<'a>(
        postgres_pool: &Pool<Postgres>,
        new_post: Post,
    ) -> Result<Self, AppError> {
        let post = query_as!(Post, r#"insert into post (date, slug, title, series, categories, markdown, post_snippet, series_snippet, published, featured) values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) returning id as "post_id?", date, slug, title, series, categories, markdown, post_snippet, series_snippet, published, featured, created_at as "post_created_at?", updated_at as "post_updated_at?""#, &new_post.date, &new_post.slug, &new_post.title, &new_post.series, &new_post.categories, &new_post.markdown, &new_post.post_snippet, &new_post.series_snippet, &new_post.published, &new_post.featured).fetch_one(postgres_pool).await?;

        Ok(post)
    }

    pub async fn update_post_postgres<'a>(
        postgres_pool: &Pool<Postgres>,
        post_id: &str,
        updated_post: Post,
    ) -> Result<(), AppError> {
        let id = uuid::Uuid::parse_str(&post_id).unwrap();

        // let post = query_as!(Post, r#"update post SET title = $2, series = $3, categories = $4, markdown = $5, date = $6 where id = $1 returning id as "post_id?", date, slug, title, series, categories, markdown, published, created_at as "post_created_at?", updated_at as "post_updated_at?""#, &id, &updated_post.title, &updated_post.series, &updated_post.categories, &updated_post.markdown, &updated_post.date).fetch_one(postgres_pool).await?;

        let _ = query_as!(Post, r#"update post set date = $2, title = $3, slug = $4, series = $5, categories = $6, markdown = $7, post_snippet = $8, series_snippet = $9, published = $10, featured = $11 where id = $1"#, &id, &updated_post.date, &updated_post.title, &updated_post.slug, &updated_post.series, &updated_post.categories, &updated_post.markdown, &updated_post.post_snippet, &updated_post.series_snippet, &updated_post.published, &updated_post.featured).execute(postgres_pool).await?;

        Ok(())
    }

    // pub async fn delete_post_postgres<'a>(
    //     postgres_con: PooledConnection<'a, PostgresConnectionManager<MakeTlsConnector>>,
    //     post_id: &str,
    // ) -> Result<(), AppError> {
    //     let id = uuid::Uuid::parse_str(&post_id).unwrap();

    //     let _ = postgres_con
    //         .query_one("DELETE FROM post WHERE id = $1", &[&id])
    //         .await;

    //     Ok(())
    // }
}
