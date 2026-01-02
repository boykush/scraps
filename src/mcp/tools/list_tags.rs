use crate::usecase::tag::list::usecase::ListTagUsecase;
use rmcp::model::ErrorCode;
use rmcp::model::{CallToolResult, Content};
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer};
use serde_json::json;
use std::path::PathBuf;

pub async fn list_tags(
    scraps_dir: &PathBuf,
    _context: RequestContext<RoleServer>,
) -> Result<CallToolResult, ErrorData> {
    // Create tag usecase
    let tag_usecase = ListTagUsecase::new(scraps_dir);

    // Execute tag listing
    let (tags, backlinks_map) = tag_usecase
        .execute()
        .map_err(|e| ErrorData::new(ErrorCode(-32004), format!("List tags failed: {e}"), None))?;

    // Convert results to JSON
    let mut tags_with_backlinks: Vec<_> = tags
        .into_iter()
        .map(|tag| {
            let backlinks_count = backlinks_map.get(&tag.title().clone().into()).len();
            json!({
                "title": tag.title().to_string(),
                "backlinks_count": backlinks_count
            })
        })
        .collect();

    // Sort by backlinks count (descending)
    tags_with_backlinks.sort_by(|a, b| {
        let count_a = a["backlinks_count"].as_u64().unwrap_or(0);
        let count_b = b["backlinks_count"].as_u64().unwrap_or(0);
        count_b.cmp(&count_a)
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string(&tags_with_backlinks).unwrap(),
    )]))
}
