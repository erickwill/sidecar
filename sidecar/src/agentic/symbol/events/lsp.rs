//! Contains the LSP signal which might be sent from the editor
//! For now, its just the diagnostics when we detect a change in the editor

use crate::chunking::text_document::Range;

#[derive(Debug, Clone)]
pub struct LSPDiagnostiErrorMetadata {
    associated_files: Option<Vec<String>>,
    quick_fix_labels: Option<Vec<String>>,
    parameter_hints: Option<Vec<String>>,
}

impl LSPDiagnostiErrorMetadata {
    fn new(
        associated_files: Option<Vec<String>>,
        quick_fix_labels: Option<Vec<String>>,
        parameter_hints: Option<Vec<String>>,
    ) -> Self {
        Self {
            associated_files,
            quick_fix_labels,
            parameter_hints,
        }
    }

    fn _set_associated_files(mut self, associated_files: Vec<String>) -> Self {
        self.associated_files = Some(
            self.associated_files
                .map(|mut self_associated_files| {
                    self_associated_files.extend(associated_files.clone());
                    self_associated_files
                })
                .unwrap_or(associated_files),
        );
        self
    }

    fn _set_quick_fix_labels(mut self, quick_fix_labels: Vec<String>) -> Self {
        self.quick_fix_labels = Some(
            self.quick_fix_labels
                .map(|mut self_quick_fix_labels| {
                    self_quick_fix_labels.extend(quick_fix_labels.clone());
                    self_quick_fix_labels
                })
                .unwrap_or(quick_fix_labels),
        );
        self
    }

    fn _set_parameter_hints(mut self, parameter_hints: Vec<String>) -> Self {
        self.parameter_hints = Some(
            self.parameter_hints
                .map(|mut self_parameter_hints| {
                    self_parameter_hints.extend(parameter_hints.clone());
                    self_parameter_hints
                })
                .unwrap_or(parameter_hints),
        );
        self
    }
}

#[derive(Debug, Clone)]
pub struct LSPDiagnosticError {
    range: Range,
    snippet: String,
    fs_file_path: String,
    diagnostic: String,
    metadata: LSPDiagnostiErrorMetadata,
}

impl LSPDiagnosticError {
    pub fn new(
        range: Range,
        snippet: String,
        fs_file_path: String,
        diagnostic: String,
        quick_fix_labels: Option<Vec<String>>,
        parameter_hints: Option<Vec<String>>,
    ) -> Self {
        Self {
            range,
            snippet,
            fs_file_path,
            diagnostic,
            metadata: LSPDiagnostiErrorMetadata::new(None, quick_fix_labels, parameter_hints),
        }
    }

    pub fn range(&self) -> &Range {
        &self.range
    }

    pub fn diagnostic_message(&self) -> &str {
        &self.diagnostic
    }

    pub fn fs_file_path(&self) -> &str {
        &self.fs_file_path
    }

    pub fn snippet(&self) -> &str {
        &self.snippet
    }

    pub fn associated_files(&self) -> Option<&Vec<String>> {
        self.metadata.associated_files.as_ref()
    }

    pub fn quick_fix_labels(&self) -> Option<&Vec<String>> {
        self.metadata.quick_fix_labels.as_ref()
    }

    pub fn parameter_hints(&self) -> Option<&Vec<String>> {
        self.metadata.parameter_hints.as_ref()
    }
}

/// Contains the different lsp signals which we get from the editor
/// instead of being poll based we can get a push event over here
pub enum LSPSignal {
    Diagnostics(Vec<LSPDiagnosticError>),
}

impl LSPSignal {
    pub fn diagnostics(diagnostics: Vec<LSPDiagnosticError>) -> Self {
        LSPSignal::Diagnostics(diagnostics)
    }
}
