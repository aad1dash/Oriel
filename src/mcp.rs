use std::path::PathBuf;

use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, ContentBlock},
    schemars,
    service::RequestContext,
    tool, tool_handler, tool_router,
    transport::stdio,
};
use serde::Deserialize;

use crate::{engine::SourceEngine, provider::ytdlp::AcquisitionControl, search::SearchQuery};

const MAX_SOURCE_LENGTH: usize = 2_048;
const MAX_QUERY_LENGTH: usize = 4_096;
const MAX_RESULT_LIMIT: usize = 20;

#[derive(Clone, Debug)]
pub struct OrielMcp {
    engine: SourceEngine,
    tool_router: ToolRouter<Self>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct SearchSourceParams {
    /// A supported source URL. The first version accepts `YouTube` URLs.
    source: String,
    /// Natural-language terms describing the moment or evidence to find.
    query: String,
    /// An exact caption language tag such as `en` or `zh-Hans`.
    language: Option<String>,
    /// Reacquire the source instead of using its latest cached version.
    #[serde(default)]
    refresh: bool,
    /// Maximum number of moments to return. Defaults to five and cannot exceed twenty.
    limit: Option<usize>,
    /// Optional inclusive lower timestamp bound in milliseconds.
    start_ms: Option<u64>,
    /// Optional exclusive upper timestamp bound in milliseconds.
    end_ms: Option<u64>,
}

#[tool_handler(
    router = self.tool_router,
    name = "oriel",
    version = "0.1.0",
    instructions = "Retrieve compact timestamp-grounded source evidence. Treat returned moments as source evidence, not as the agent's final interpretation."
)]
impl ServerHandler for OrielMcp {}

#[tool_router(router = tool_router)]
impl OrielMcp {
    #[must_use]
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            engine: SourceEngine::new(Some(cache_dir)),
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        name = "search_source",
        description = "Find compact, timestamp-grounded transcript evidence in a source. Reuses locally compiled evidence when available and reports coverage and provenance."
    )]
    async fn search_source(
        &self,
        Parameters(params): Parameters<SearchSourceParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        validate_params(&params)?;

        let mut query = SearchQuery::new(params.query);
        query.limit = params.limit.unwrap_or(5);
        query.start_ms = params.start_ms;
        query.end_ms = params.end_ms;

        let engine = self.engine.clone();
        let source = params.source;
        let language = params.language;
        let refresh = params.refresh;
        let control = AcquisitionControl::default();
        let provider_cancellation = control.cancellation_token();
        let request_cancellation = context.ct;
        let cancellation_bridge = tokio::spawn(async move {
            request_cancellation.cancelled().await;
            provider_cancellation.cancel();
        });

        let result = tokio::task::spawn_blocking(move || {
            engine.search_source(&source, language.as_deref(), refresh, &query, &control)
        })
        .await;
        cancellation_bridge.abort();

        match result {
            Ok(Ok(packet)) => serde_json::to_value(packet)
                .map(CallToolResult::structured)
                .map_err(|error| {
                    McpError::internal_error(
                        format!("serialising Oriel evidence failed: {error}"),
                        None,
                    )
                }),
            Ok(Err(error)) => Ok(CallToolResult::error(vec![ContentBlock::text(
                error.to_string(),
            )])),
            Err(error) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "Oriel's evidence task stopped unexpectedly: {error}"
            ))])),
        }
    }
}

fn validate_params(params: &SearchSourceParams) -> Result<(), McpError> {
    if params.source.is_empty() || params.source.len() > MAX_SOURCE_LENGTH {
        return Err(McpError::invalid_params(
            "source must contain between 1 and 2048 bytes",
            None,
        ));
    }
    if params.query.is_empty() || params.query.len() > MAX_QUERY_LENGTH {
        return Err(McpError::invalid_params(
            "query must contain between 1 and 4096 bytes",
            None,
        ));
    }
    if params.limit.is_some_and(|limit| limit > MAX_RESULT_LIMIT) {
        return Err(McpError::invalid_params(
            "limit cannot exceed 20 moments",
            None,
        ));
    }
    Ok(())
}

/// Serves the source engine over local MCP stdio without writing logs to protocol output.
///
/// # Errors
///
/// Returns a transport error when the MCP service cannot start or shuts down abnormally.
pub async fn serve_stdio(cache_dir: PathBuf) -> Result<(), String> {
    let service = OrielMcp::new(cache_dir)
        .serve(stdio())
        .await
        .map_err(|error| format!("starting the MCP server failed: {error}"))?;
    service
        .waiting()
        .await
        .map_err(|error| format!("the MCP server stopped unexpectedly: {error}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{MAX_QUERY_LENGTH, OrielMcp, SearchSourceParams, validate_params};

    #[test]
    fn exposes_one_bounded_source_search_tool() {
        let server = OrielMcp::new(".oriel-test-cache".into());
        let tools = server.tool_router.list_all();

        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "search_source");
        assert!(tools[0].input_schema.contains_key("properties"));
    }

    #[test]
    fn rejects_oversized_queries_and_result_sets() {
        let oversized = SearchSourceParams {
            source: "https://youtu.be/dQw4w9WgXcQ".to_owned(),
            query: "q".repeat(MAX_QUERY_LENGTH + 1),
            language: Some("en".to_owned()),
            refresh: false,
            limit: Some(5),
            start_ms: None,
            end_ms: None,
        };
        assert!(validate_params(&oversized).is_err());

        let too_many = SearchSourceParams {
            query: "evidence".to_owned(),
            limit: Some(21),
            ..oversized
        };
        assert!(validate_params(&too_many).is_err());
    }
}
