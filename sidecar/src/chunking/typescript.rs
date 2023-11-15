use crate::chunking::languages::TSLanguageConfig;

pub fn typescript_language_config() -> TSLanguageConfig {
    TSLanguageConfig {
        language_ids: &["Typescript", "TSX", "typescript", "tsx"],
        file_extensions: &["ts", "tsx", "jsx", "mjs"],
        grammar: tree_sitter_typescript::language_tsx,
        namespaces: vec![
            "constant",
            "variable",
            "property",
            "parameter",
            // functions
            "function",
            "method",
            "generator",
            // types
            "alias",
            "enum",
            "enumerator",
            "class",
            "interface",
            // misc.
            "label",
        ]
        .into_iter()
        .map(|s| s.to_owned())
        .collect(),
        documentation_query: vec!["((comment) @comment
        (#match? @comment \"^\\\\/\\\\*\\\\*\")) @docComment"
            .to_owned()],
        function_query: vec!["[
            (function
                name: (identifier)? @identifier
                parameters: (formal_parameters)? @parameters
                return_type: (type_annotation)? @return_type
                body: (statement_block) @body)
            (function_declaration
                name: (identifier)? @identifier
                parameters: (formal_parameters)? @parameters
                return_type: (type_annotation)? @return_type
                body: (statement_block) @body)
            (generator_function
                name: (identifier)? @identifier
                parameters: (formal_parameters)? @parameters
                return_type: (type_annotation)? @return_type
                body: (statement_block) @body)
            (generator_function_declaration
                name: (identifier)? @identifier
                parameters: (formal_parameters)? @parameters
                return_type: (type_annotation)? @return_type
                body: (statement_block) @body)
            (method_definition
                name: (property_identifier)? @identifier
                parameters: (formal_parameters)? @parameters
                return_type: (type_annotation)? @return_type
                body: (statement_block) @body)
            (arrow_function
                body: (statement_block) @body
                parameters: (formal_parameters)? @parameters
                return_type: (type_annotation)? @return_type)
            ] @function"
            .to_owned()],
        construct_types: vec![
            "program",
            "interface_declaration",
            "class_declaration",
            "function_declaration",
            "function",
            "type_alias_declaration",
            "method_definition",
        ]
        .into_iter()
        .map(|s| s.to_owned())
        .collect(),
        expression_statements: vec![
            "lexical_declaration",
            "expression_statement",
            "public_field_definition",
        ]
        .into_iter()
        .map(|s| s.to_owned())
        .collect(),
        class_query: vec![
            "[(abstract_class_declaration name: (type_identifier)? @identifier) (class_declaration name: (type_identifier)? @identifier)] @class_declaration"
                .to_owned(),
        ],
    }
}