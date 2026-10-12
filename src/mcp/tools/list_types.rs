use crate::ir::loader;
use crate::usecase::scrap_type::list::usecase::ListTypeUsecase;
use rmcp::model::ErrorCode;
use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct TypeJson {
    #[serde(rename = "type")]
    pub name: String,
    pub scraps_count: usize,
}

#[derive(Debug, Serialize)]
pub struct ListTypesResponse {
    pub results: Vec<TypeJson>,
    pub count: usize,
    pub next: String,
}

pub async fn list_types(
    scraps_dir: &Path,
    exclude_dirs: &[std::path::PathBuf],
    _context: RequestContext<RoleServer>,
) -> Result<CallToolResult, ErrorData> {
    let scraps = loader::load_scraps(scraps_dir, exclude_dirs).map_err(|e| {
        ErrorData::new(
            ErrorCode(-32003),
            format!("Failed to load scraps: {e}"),
            None,
        )
    })?;

    let type_usecase = ListTypeUsecase::new();

    let results = type_usecase
        .execute(&scraps)
        .map_err(|e| ErrorData::new(ErrorCode(-32004), format!("List types failed: {e}"), None))?;

    let types: Vec<TypeJson> = results
        .into_iter()
        .map(|result| TypeJson {
            name: result.name,
            scraps_count: result.scraps_count,
        })
        .collect();

    let count = types.len();
    let next = if count == 0 {
        "No scrap declares a type. Browse by topic with list_tags, or search content with search_scraps."
    } else {
        "Expand a type into its scraps with lookup_type_scraps {type}."
    };
    let response = ListTypesResponse {
        results: types,
        count,
        next: next.to_string(),
    };

    Ok(CallToolResult::success(vec![ContentBlock::text(
        serde_json::to_string(&response).map_err(|e| {
            ErrorData::new(
                ErrorCode(-32005),
                format!("JSON serialization failed: {e}"),
                None,
            )
        })?,
    )]))
}
