use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::chunking::languages::TSLanguageParsing;

use super::error::RepoMapError;

use super::file::errors::FileError;
use super::file::git::GitWalker;
use futures::{stream, StreamExt};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tag {
    pub rel_fname: PathBuf,
    pub fname: PathBuf,
    pub line: usize,
    pub name: String,
    pub kind: TagKind,
}

impl Tag {
    pub fn new(
        rel_fname: PathBuf,
        fname: PathBuf,
        line: usize,
        name: String,
        kind: TagKind,
    ) -> Self {
        Self {
            rel_fname,
            fname,
            line,
            name,
            kind,
        }
    }

    /// Using this to generate a dummy tag
    pub fn dummy() -> Self {
        Self {
            rel_fname: PathBuf::new(),
            fname: PathBuf::new(),
            line: 0,
            name: "".to_owned(),
            kind: TagKind::Definition,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TagKind {
    Definition,
    Reference,
}

/// An index structure for managing tags across multiple files.
pub struct TagIndex {
    /// Maps tag names to the set of file paths where the tag is defined.
    ///
    /// Useful for answering: "In which files is tag X defined?"
    pub defines: HashMap<String, HashSet<PathBuf>>,

    /// Maps tag names to a list of file paths where the tag is referenced.
    ///
    /// Allows duplicates to accommodate multiple references to the same definition.
    pub references: HashMap<String, Vec<PathBuf>>,

    /// Maps (file path, tag name) pairs to a set of tag definitions.
    ///
    /// Useful for answering: "What are the details of tag X in file Y?"
    ///
    /// Needs to be a HashSet<Tag> due to function overloading where multiple functions share the same name but have different parameters
    pub definitions: HashMap<(PathBuf, String), HashSet<Tag>>,

    /// A set of commonly used tags across all files.
    pub common_tags: HashSet<String>,

    /// Maps file paths to the set of tags defined in the file.
    ///
    /// Useful for answering: "What are the tags defined in file X?"
    pub file_to_tags: HashMap<PathBuf, HashSet<String>>,
}

impl TagIndex {
    pub fn new() -> Self {
        Self {
            defines: HashMap::new(),
            references: HashMap::new(),
            definitions: HashMap::new(),
            common_tags: HashSet::new(),
            file_to_tags: HashMap::new(),
        }
    }

    pub fn get_files(root: &Path) -> Result<HashMap<String, Vec<u8>>, FileError> {
        let git_walker = GitWalker {};
        git_walker.read_files(root)
    }

    pub async fn generate_from_files(&mut self, files: HashMap<String, Vec<u8>>) {
        self.generate_tag_index(files).await;
    }

    pub async fn from_path(path: &Path) -> Self {
        let mut index = TagIndex::new();
        let files = TagIndex::get_files(path).unwrap();

        index.generate_tag_index(files).await;

        index
    }

    pub fn post_process_tags(&mut self) {
        self.process_empty_references();
        self.process_common_tags();
    }

    pub fn add_tag(&mut self, tag: Tag, rel_path: &PathBuf) {
        match tag.kind {
            TagKind::Definition => {
                self.defines
                    .entry(tag.name.clone())
                    .or_default()
                    .insert(rel_path.clone());
                self.definitions
                    .entry((rel_path.clone(), tag.name.clone()))
                    .or_default()
                    .insert(tag.clone());

                self.file_to_tags
                    .entry(rel_path.clone())
                    .or_default()
                    .insert(tag.name.clone());
            }
            TagKind::Reference => {
                self.references
                    .entry(tag.name.clone())
                    .or_default()
                    .push(rel_path.clone());

                self.file_to_tags
                    .entry(rel_path.clone())
                    .or_default()
                    .insert(tag.name.clone());
            }
        }
    }

    pub fn process_empty_references(&mut self) {
        if self.references.is_empty() {
            self.references = self
                .defines
                .iter()
                .map(|(k, v)| (k.clone(), v.iter().cloned().collect::<Vec<PathBuf>>()))
                .collect();
        }
    }

    pub fn process_common_tags(&mut self) {
        self.common_tags = self
            .defines
            .keys()
            .filter_map(|key| match self.references.contains_key(key) {
                true => Some(key.clone()),
                false => None,
            })
            .collect();
    }

    pub fn debug_print(&self) {
        println!("==========Defines==========");
        self.defines.iter().for_each(|(key, set)| {
            println!("Key {}, Set: {:?}", key, set);
        });

        println!("==========Definitions==========");
        self.definitions
            .iter()
            .for_each(|((pathbuf, tag_name), set)| {
                println!("Key {:?}, Set: {:?}", (pathbuf, tag_name), set);
            });

        println!("==========References==========");
        self.references.iter().for_each(|(tag_name, paths)| {
            println!("Tag: {}, Paths: {:?}", tag_name, paths);
        });

        println!("==========Common Tags==========");
        self.common_tags.iter().for_each(|tag| {
            println!(
                "Common Tag: {}\n(defined in: {:?}, referenced in: {:?})",
                tag, &self.defines[tag], &self.references[tag]
            );
        });
    }

    async fn generate_tag_index(&mut self, files: HashMap<String, Vec<u8>>) {
        let ts_parsing = Arc::new(TSLanguageParsing::init());
        let _ = stream::iter(
            files
                .into_iter()
                .map(|(file, _)| (file, ts_parsing.clone())),
        )
        .map(|(file, ts_parsing)| async {
            self.generate_tags_for_file(&file, ts_parsing)
                .await
                .map(|tags| (tags, file))
                .ok()
        })
        .buffer_unordered(10000)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .filter_map(|s| s)
        .for_each(|(tags, file)| {
            let file_ref = &file;
            tags.into_iter().for_each(|tag| {
                self.add_tag(tag, &PathBuf::from(file_ref));
            });
        });

        self.post_process_tags();
    }

    async fn generate_tags_for_file(
        &self,
        fname: &str,
        ts_parsing: Arc<TSLanguageParsing>,
    ) -> Result<Vec<Tag>, RepoMapError> {
        let rel_path = self.get_rel_fname(&PathBuf::from(fname));
        let config = ts_parsing.for_file_path(fname).ok_or_else(|| {
            RepoMapError::ParseError(format!("Language configuration not found for: {}", fname,))
        });
        let content = tokio::fs::read(fname).await;
        if let Err(_) = content {
            return Err(RepoMapError::IoError);
        }
        let content = content.expect("if let Err to hold");
        if let Ok(config) = config {
            let tags = config
                .get_tags(&PathBuf::from(fname), &rel_path, content)
                .await;
            Ok(tags)
        } else {
            Ok(vec![])
        }
    }

    pub fn get_tags_for_file(&self, file_name: &Path) -> Option<Vec<String>> {
        let tag_names = self.file_to_tags.get(file_name)?;
        Some(tag_names.iter().cloned().collect())
    }

    pub fn print_file_to_tag_keys(&self) {
        self.file_to_tags.keys().for_each(|key| {
            println!("{}", key.display());
        });
    }

    fn get_rel_fname(&self, fname: &PathBuf) -> PathBuf {
        let self_root = env!("CARGO_MANIFEST_DIR").to_string();
        fname
            .strip_prefix(&self_root)
            .unwrap_or(fname)
            .to_path_buf()
    }

    // Add methods to query the index as needed
}
