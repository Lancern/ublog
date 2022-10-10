use std::error::Error;
use std::sync::Arc;

use rss::{
    CategoryBuilder as RssCategoryBuilder, Channel as RssChannel,
    ChannelBuilder as RssChannelBuilder, ItemBuilder as RssItemBuilder,
};
use time::format_description::well_known::Rfc2822;
use time::OffsetDateTime;
use ublog_data::models::Post;
use ublog_data::storage::Pagination;

use crate::server::config::SiteConfig;
use crate::server::ServerContext;

pub(super) async fn compute_rss(ctx: Arc<ServerContext>) -> Result<RssChannel, Box<dyn Error>> {
    let pagination = Pagination::from_page_and_size(1, RSS_POSTS);
    let posts = ctx.db.get_posts(false, &pagination).await.map_err(|err| {
        spdlog::error!("Get posts list from database for RSS failed: {}", err);
        Box::<dyn Error>::from(err)
    })?;

    let mut channel_builder = RssChannelBuilder::default();

    channel_builder
        .title(ctx.site.title.clone())
        .link(ctx.site.url.clone())
        .copyright(ctx.site.copyright.clone())
        .last_build_date(OffsetDateTime::now_utc().format(&Rfc2822).unwrap())
        .generator(String::from("ublog"));

    for p in &posts.objects {
        let item = RssItemBuilder::default()
            .title(p.title.clone())
            .link(create_post_url(&ctx.site, p))
            .author(ctx.site.owner.clone())
            .category(
                RssCategoryBuilder::default()
                    .name(p.category.clone())
                    .build(),
            )
            .pub_date(
                OffsetDateTime::from_unix_timestamp(p.update_timestamp)
                    .unwrap()
                    .format(&Rfc2822)
                    .unwrap(),
            )
            .build();

        channel_builder.item(item);
    }

    let channel = channel_builder.build();
    Ok(channel)
}

fn create_post_url(site: &SiteConfig, post: &Post) -> String {
    site.post_url_template.replace("${slug}", &post.slug)
}

const RSS_POSTS: usize = 50;
