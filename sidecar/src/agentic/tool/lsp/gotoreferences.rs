use async_trait::async_trait;

use crate::{
    agentic::tool::{errors::ToolError, input::ToolInput, output::ToolOutput, r#type::Tool},
    chunking::text_document::{Position, Range},
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GoToReferencesRequest {
    fs_file_path: String,
    position: Position,
    editor_url: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReferenceLocation {
    fs_file_path: String,
    range: Range,
}

impl ReferenceLocation {
    pub fn fs_file_path(&self) -> &str {
        &self.fs_file_path
    }

    pub fn range(&self) -> &Range {
        &self.range
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GoToReferencesResponse {
    reference_locations: Vec<ReferenceLocation>,
}

impl GoToReferencesResponse {
    pub fn locations(self) -> Vec<ReferenceLocation> {
        self.reference_locations
    }

    /// filters out the locations which are pointing to the same location where we
    /// are checking for the references
    pub fn filter_out_same_position_location(
        mut self,
        fs_file_path: &str,
        position: &Position,
    ) -> Self {
        let range_to_check = Range::new(position.clone(), position.clone());
        self.reference_locations = self
            .reference_locations
            .into_iter()
            .filter(|location| {
                !(location.fs_file_path == fs_file_path
                    && location.range().contains_check_line_column(&range_to_check))
            })
            .collect::<Vec<_>>();
        self
    }
}

impl GoToReferencesRequest {
    pub fn new(fs_file_path: String, position: Position, editor_url: String) -> Self {
        Self {
            fs_file_path,
            position,
            editor_url,
        }
    }
}

pub struct LSPGoToReferences {
    client: reqwest::Client,
}

impl LSPGoToReferences {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Tool for LSPGoToReferences {
    async fn invoke(&self, input: ToolInput) -> Result<ToolOutput, ToolError> {
        let context = input.reference_request()?;
        let editor_endpoint = context.editor_url.to_owned() + "/go_to_references";
        let response = self
            .client
            .post(editor_endpoint)
            .body(serde_json::to_string(&context).map_err(|_e| ToolError::SerdeConversionFailed)?)
            .send()
            .await
            .map_err(|_e| ToolError::ErrorCommunicatingWithEditor)?;
        let response: GoToReferencesResponse = response
            .json()
            .await
            .map_err(|_e| ToolError::SerdeConversionFailed)?;
        Ok(ToolOutput::go_to_reference(response))
    }
}
