//! We are going to cache the tree sitter trees for the current file
//! and use that for keeping the context hot and ready when we have
//! to insert some text into the middle of the file, making sure that
//! we can easily keep track of the errors and the changes generated by inserting
//! the text

use std::sync::Arc;

use dashmap::DashMap;

use crate::chunking::editor_parsing::EditorParsing;

pub struct TreeSitterCache {
    pub cache: DashMap<String, tree_sitter::Tree>,
    editor_parsing: Arc<EditorParsing>,
}

impl TreeSitterCache {
    pub fn new(editor_parsing: Arc<EditorParsing>) -> Self {
        Self {
            cache: DashMap::new(),
            editor_parsing,
        }
    }

    pub fn insert_text(&self, key: String, language: String, text: String) {
        let language_config = self.editor_parsing.ts_language_config(&language);
        if let None = language_config {
            return;
        }
        let language_config = language_config.expect("if let None to hold");
        let mut parser = tree_sitter::Parser::new();
        let _ = parser.set_language((language_config.grammar)());
        let tree = tree_sitter::Parser::new().parse(text.as_bytes(), None);
        if let Some(tree) = tree {
            self.cache.insert(key, tree);
        }
    }

    pub fn update_tree(&self, key: String, new_text: &str, language: &str) {
        let updated_tree = self.cache.get_mut(&key);
        if let Some(mut tree) = updated_tree {
            // Now we create an updated tree by using the previous tree and
            // parsing it with the new text
            let mut parser = tree_sitter::Parser::new();
            let language_config = self.editor_parsing.ts_language_config(&language);
            if let None = language_config {
                return;
            }
            let language_config = language_config.expect("if let None to hold");
            let _ = parser.set_language((language_config.grammar)());
            let new_tree = parser.parse(new_text.as_bytes(), Some(&tree));
            if let Some(new_tree) = new_tree {
                *tree = new_tree;
            }
        } else {
            self.insert_text(key, language.to_string(), new_text.to_string());
        }
    }
}
