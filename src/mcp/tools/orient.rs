use crate::input::file::read_scraps;
use crate::usecase::tag::list::usecase::ListTagUsecase;
use rmcp::model::ErrorCode;
use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer};
use serde_json::json;
use std::path::Path;

// Head of the topic map only; the full list stays list_tags' job.
const TOP_TAGS_LIMIT: usize = 10;

pub async fn orient(
    scraps_dir: &Path,
    exclude_dirs: &[std::path::PathBuf],
    _context: RequestContext<RoleServer>,
) -> Result<CallToolResult, ErrorData> {
    // Load scraps from directory
    let scraps = read_scraps::to_all_scraps(scraps_dir, exclude_dirs).map_err(|e| {
        ErrorData::new(
            ErrorCode(-32003),
            format!("Failed to load scraps: {e}"),
            None,
        )
    })?;

    let mut contexts: Vec<String> = scraps
        .iter()
        .filter_map(|s| s.ctx().as_ref().map(|c| c.to_string()))
        .collect();
    contexts.sort();
    contexts.dedup();

    // Create tag usecase
    let tag_usecase = ListTagUsecase::new();

    let (tags, backlinks_map) = tag_usecase
        .execute(&scraps)
        .map_err(|e| ErrorData::new(ErrorCode(-32004), format!("List tags failed: {e}"), None))?;

    let tag_count = tags.len();
    let mut ranked: Vec<(String, usize)> = tags
        .into_iter()
        .map(|tag| {
            let backlinks_count = backlinks_map.get_tag(&tag).len();
            (tag.to_string(), backlinks_count)
        })
        .collect();
    // Tie-break by name so the head of the topic map is stable.
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let top_tags: Vec<_> = ranked
        .into_iter()
        .take(TOP_TAGS_LIMIT)
        .map(|(title, backlinks_count)| {
            json!({
                "title": title,
                "backlinks_count": backlinks_count
            })
        })
        .collect();

    let scrap_count = scraps.len();
    let next = if scrap_count == 0 {
        "The wiki has no scraps yet."
    } else {
        "Search content with search_scraps, or expand a top tag with lookup_tag_backlinks {tag}; the full tag list is list_tags."
    };

    let response = json!({
        "scrap_count": scrap_count,
        "tag_count": tag_count,
        "contexts": contexts,
        "top_tags": top_tags,
        "next": next,
    });

    Ok(CallToolResult::success(vec![ContentBlock::text(
        serde_json::to_string(&response).unwrap(),
    )]))
}
