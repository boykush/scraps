use crate::input::file::read_scraps;
use crate::mcp::json::scrap::ScrapKeyJson;
use crate::usecase::todo::usecase::{status_label, StatusFilter, TodoUsecase};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ErrorCode;
use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::schemars::JsonSchema;
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Status filter for task items
#[derive(Debug, Clone, Copy, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum TodoStatus {
    /// Unchecked `- [ ]` items (default)
    #[default]
    Open,
    /// Checked `- [x]` items
    Done,
    /// Deferred `- [-]` items
    Deferred,
    /// Every task item, whatever its status
    All,
}

impl From<TodoStatus> for StatusFilter {
    fn from(status: TodoStatus) -> Self {
        match status {
            TodoStatus::Open => StatusFilter::Open,
            TodoStatus::Done => StatusFilter::Done,
            TodoStatus::Deferred => StatusFilter::Deferred,
            TodoStatus::All => StatusFilter::All,
        }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct ListTodosRequest {
    /// Status filter: "open" (default), "done", "deferred", or "all"
    pub status: Option<TodoStatus>,
}

#[derive(Debug, Serialize)]
pub struct TodoItemJson {
    pub scrap: ScrapKeyJson,
    pub status: String,
    pub text: String,
    pub line: usize,
}

#[derive(Debug, Serialize)]
pub struct ListTodosResponse {
    pub results: Vec<TodoItemJson>,
    pub count: usize,
    pub next: String,
}

pub async fn list_todos(
    scraps_dir: &Path,
    exclude_dirs: &[std::path::PathBuf],
    _context: RequestContext<RoleServer>,
    Parameters(request): Parameters<ListTodosRequest>,
) -> Result<CallToolResult, ErrorData> {
    let scraps = read_scraps::to_all_scraps(scraps_dir, exclude_dirs).map_err(|e| {
        ErrorData::new(
            ErrorCode(-32003),
            format!("Failed to load scraps: {e}"),
            None,
        )
    })?;

    let todo_usecase = TodoUsecase::new();

    let status_filter = request.status.unwrap_or_default().into();
    let results = todo_usecase
        .execute(&scraps, status_filter)
        .map_err(|e| ErrorData::new(ErrorCode(-32004), format!("List todos failed: {e}"), None))?;

    let items: Vec<TodoItemJson> = results
        .into_iter()
        .map(|result| TodoItemJson {
            scrap: ScrapKeyJson {
                title: result.title.to_string(),
                ctx: result.ctx.map(|c| c.to_string()),
            },
            status: status_label(&result.status).to_string(),
            text: result.text,
            line: result.line,
        })
        .collect();

    let count = items.len();
    let next = if count == 0 {
        "No task items with that status. Widen it with status 'all', or look for the work in prose with search_scraps."
    } else {
        "Read the scrap a task sits in with get_scrap {title, ctx}."
    };
    let response = ListTodosResponse {
        results: items,
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
