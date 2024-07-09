use crate::{
    agentic::tool::{errors::ToolError, input::ToolInput, output::ToolOutput, r#type::Tool},
    chunking::text_document::Range,
};
use async_trait::async_trait;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LSPGrepSymbolInCodebaseRequest {
    editor_url: String,
    search_string: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LocationInformation {
    name: String,
    range: Range,
    fs_file_path: String,
}

impl LocationInformation {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn fs_file_path(&self) -> &str {
        &self.fs_file_path
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LSPGrepSymbolInCodebaseResponse {
    locations: Vec<LocationInformation>,
}

impl LSPGrepSymbolInCodebaseResponse {
    pub fn locations(&self) -> &[LocationInformation] {
        self.locations.as_slice()
    }
}

impl LSPGrepSymbolInCodebaseRequest {
    pub fn new(editor_url: String, search_string: String) -> Self {
        Self {
            editor_url,
            search_string,
        }
    }
}

pub struct GrepSymbolInCodebase {
    client: reqwest::Client,
}

impl GrepSymbolInCodebase {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Tool for GrepSymbolInCodebase {
    async fn invoke(&self, input: ToolInput) -> Result<ToolOutput, ToolError> {
        let context = input.grep_symbol_in_codebase()?;
        let editor_endpoint = context.editor_url.to_owned() + "/symbol_search";
        let response = self
            .client
            .post(editor_endpoint)
            .body(serde_json::to_string(&context).map_err(|_e| ToolError::SerdeConversionFailed)?)
            .send()
            .await
            .map_err(|_e| ToolError::ErrorCommunicatingWithEditor)?;
        let response: LSPGrepSymbolInCodebaseResponse = response
            .json()
            .await
            .map_err(|_e| ToolError::SerdeConversionFailed)?;
        Ok(ToolOutput::lsp_symbol_search_information(response))
    }
}
