use std::{path::PathBuf, sync::Arc};

use llm_client::{
    broker::LLMBroker,
    clients::types::LLMType,
    config::LLMBrokerConfiguration,
    provider::{GoogleAIStudioKey, LLMProvider, LLMProviderAPIKeys},
};
use sidecar::{
    agentic::{
        symbol::{identifier::LLMProperties, tool_box::ToolBox},
        tool::{
            broker::{ToolBroker, ToolBrokerConfiguration},
            code_edit::models::broker::CodeEditBroker,
        },
    },
    chunking::{editor_parsing::EditorParsing, languages::TSLanguageParsing},
    inline_completion::symbols_tracker::SymbolTrackerInline,
};

fn default_index_dir() -> PathBuf {
    match directories::ProjectDirs::from("ai", "codestory", "sidecar") {
        Some(dirs) => dirs.data_dir().to_owned(),
        None => "codestory_sidecar".into(),
    }
}

#[tokio::main]
async fn main() {
    // we want to grab the implementations of the symbols over here which we are
    // interested in
    let editor_url = "http://localhost:42423".to_owned();
    let editor_parsing = Arc::new(EditorParsing::default());
    let symbol_broker = Arc::new(SymbolTrackerInline::new(editor_parsing.clone()));
    let tool_broker = Arc::new(ToolBroker::new(
        Arc::new(
            LLMBroker::new(LLMBrokerConfiguration::new(default_index_dir()))
                .await
                .expect("to initialize properly"),
        ),
        Arc::new(CodeEditBroker::new()),
        symbol_broker.clone(),
        Arc::new(TSLanguageParsing::init()),
        ToolBrokerConfiguration::new(None, true),
        LLMProperties::new(
            LLMType::GeminiPro,
            LLMProvider::GoogleAIStudio,
            LLMProviderAPIKeys::GoogleAIStudio(GoogleAIStudioKey::new(
                "AIzaSyCMkKfNkmjF8rTOWMg53NiYmz0Zv6xbfsE".to_owned(),
            )),
        ),
    ));

    let tool_box = Arc::new(ToolBox::new(
        tool_broker,
        symbol_broker,
        editor_parsing,
        editor_url,
        "".to_owned(),
    ));

    let root_directory = "/Users/skcd/scratch/sidecar";
    let fs_file_path = "/Users/skcd/scratch/sidecar/sidecar/src/agentic/symbol/tool_box.rs";
    let output = tool_box
        .grab_changed_symbols_in_file(root_directory, fs_file_path)
        .await
        .expect("to work");

    // from here we have to go a level deeper into the sub-symbol of the symbol where
    // the changed values are present and then invoke a followup at that point
    // println!("{:?}", &output);
    // a more readable output
    output.into_iter().for_each(|(symbol_name, edits)| {
        println!(
            "symbol_name::({})::children({})",
            symbol_name,
            edits
                .into_iter()
                .map(|(symbol_to_edit, _)| symbol_to_edit.symbol_name().to_owned())
                .collect::<Vec<_>>()
                .join(",")
        );
    })
}
