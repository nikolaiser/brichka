use anyhow::Context;
use anyhow::Result;
use dashmap::DashMap;
use dashmap::DashSet;
use tokio::fs;
use std::borrow::Cow;
use std::sync::Arc;
use tower_lsp::jsonrpc;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

struct Cache {
    catalogs: DashMap<String, Catalog>,
}

impl Cache {
    pub async fn init(&self) -> Result<()> {
        let response = crate::client::uc::list_catalogs().await?;
        for catalog in response.catalogs {
            self.catalogs.insert(catalog.name.to_owned(), Catalog{ name: catalog.name, schemas: DashMap::new() });
        }
        Ok(())
    }
}

 struct Catalog {
    name: String,
    schemas: DashMap<String, Schema>
}

impl Catalog {
    pub async fn init(&self) -> Result<()> {
        let response = crate::client::uc::list_schemas(self.name.to_owned()).await?;
        for schema in response.schemas {
            self.schemas.insert(schema.name.to_owned(), Schema{ name: schema.name, catalog_name: self.name.to_owned(), tables: DashSet::new() });
        }
        Ok(())
    }
}

struct Schema {
    name: String,
    catalog_name: String,
    tables: DashSet<String>
}

impl Schema {
    pub async fn init(&self) -> Result<()> {
        let response = crate::client::uc::list_tables(self.catalog_name.to_owned(), self.name.to_owned()).await?;

        for table in response.tables {
            self.tables.insert(table.name);
        }
        Ok(())
    }
}

pub struct Backend {
    client: Client,
    cache: Arc<Cache>,
    documents: Arc<DashMap<String, String>>,
}

//TODO: Rewrite completions
impl Backend {
    fn new (client: Client) -> Self {
        Backend { client: client, cache: Arc::new(Cache{catalogs: DashMap::new()}), documents: Arc::new(DashMap::new()) }
    }


    fn get_text_before_cursor(&self, params: &CompletionParams, text: &str) -> String {
        let position = params.text_document_position.position;
        let lines: Vec<&str> = text.lines().collect();

        if position.line as usize >= lines.len() {
            return String::new();
        }

        let line = lines[position.line as usize];
        let char_offset = position.character as usize;

        if char_offset > line.len() {
            return line.to_string();
        }

        line[..char_offset].to_string()
    }

    fn parse_identifier_chain(&self, text_before_cursor: &str) -> Vec<String> {
        let trimmed = text_before_cursor.trim_end();

        let start_pos = trimmed
            .rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
            .map(|pos| pos + 1)
            .unwrap_or(0);

        let identifier = &trimmed[start_pos..];

        identifier
            .split('.')
            .map(|s| s.to_string())
            .collect()
    }


    fn complete_catalogs(&self, prefix: &str) -> Result<Vec<CompletionItem>> {
        let items: Vec<CompletionItem> = self.cache.catalogs.iter().map(|catalog|catalog.name.to_owned()).filter(|name|name.starts_with(prefix)).map(|name| CompletionItem {
                label: name.to_owned(),
                kind: Some(CompletionItemKind::MODULE),
                ..Default::default()
        }).collect();

        Ok(items)
    }

    async fn complete_schemas(&self, catalog_name: &str, prefix: &str) -> Result<Vec<CompletionItem>> {
        match self.cache.catalogs.get(catalog_name) {
            None => Ok(Vec::new()),
            Some(catalog) => {
                if catalog.schemas.is_empty() {
                    catalog.init().await?;
                }

                let items = catalog.schemas.iter().map(|schema|schema.name.to_owned()).filter(|name|name.starts_with(prefix)).map(|name| CompletionItem {
                label: name.to_owned(),
                kind: Some(CompletionItemKind::MODULE),
                ..Default::default()
        }).collect();

                Ok(items)
            },
        }
    }


    async fn complete_tables(&self, catalog_name: &str, schema_name: &str, prefix: &str) -> Result<Vec<CompletionItem>> {
        match self.cache.catalogs.get(catalog_name) {
            None => Ok(Vec::new()),
            Some(catalog) => {
                if catalog.schemas.is_empty() {
                    catalog.init().await?;
                }

                match catalog.schemas.get(schema_name) {
                   None => Ok(Vec::new()),
                   Some(schema) => {

                       if schema.tables.is_empty() {
                           schema.init().await?;
                       }

                        let items = schema.tables.iter().filter(|name|name.starts_with(prefix)).map(|name| CompletionItem {
                        label: name.to_owned(),
                        kind: Some(CompletionItemKind::CLASS),
                        ..Default::default()
                }).collect();

                        Ok(items)
                   }
                }

            },
        }
    }
}


#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        match self.cache.init().await {
            Ok(()) => {

                jsonrpc::Result::Ok(InitializeResult {
                    server_info: Some(ServerInfo {
                        name: "brichka".to_string(),
                        version: Some(env!("CARGO_PKG_VERSION").to_string()),
                    }),
                    capabilities: ServerCapabilities {
                        text_document_sync: Some(TextDocumentSyncCapability::Kind(
                            TextDocumentSyncKind::FULL,
                        )),
                        completion_provider: Some(CompletionOptions {
                            trigger_characters: Some(vec![".".to_string()]),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                })
            },
            Err(err) => {
                jsonrpc::Result::Err(jsonrpc::Error{code:jsonrpc::ErrorCode::ServerError(0), message: Cow::Owned(err.to_string()), data: None })
            }
        }

    }

    async fn shutdown(&self) -> jsonrpc::Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        let text = params.text_document.text;
        self.documents.insert(uri, text);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        if let Some(change) = params.content_changes.first() {
            self.documents.insert(uri, change.text.clone());
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        self.documents.remove(&uri);
    }

    //TODO: Rewrite
    async fn completion(&self, params: CompletionParams) -> jsonrpc::Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri.to_string();

        let text = match self.documents.get(&uri) {
            Some(entry) => entry.value().clone(),
            None => {
                self.client
                    .log_message(MessageType::WARNING, format!("Document not found: {}", uri))
                    .await;
                return Ok(None);
            }
        };

        let text_before_cursor = self.get_text_before_cursor(&params, &text);

        let parts = self.parse_identifier_chain(&text_before_cursor);

        self.client
            .log_message(
                MessageType::INFO,
                format!("Completion parts: {:?}", parts),
            )
            .await;


        let completions = match parts.len() {
            0 => {
                self.complete_catalogs("").unwrap_or_default()
            }
            1 => {
                let prefix = &parts[0];
                self.complete_catalogs(prefix).unwrap_or_default()
            }
            2 => {
                let catalog_name = &parts[0];
                let schema_prefix = &parts[1];
                self.complete_schemas(catalog_name, schema_prefix).await.unwrap_or_default()
            }
            3 => {
                let catalog_name = &parts[0];
                let schema_name = &parts[1];
                let table_prefix = &parts[2];
                self.complete_tables(catalog_name, schema_name, table_prefix)
                    .await
                    .unwrap_or_default()
            }
            _ => vec![],
        };

        if completions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(CompletionResponse::Array(completions)))
        }
    }

}


pub async fn start() -> Result<()> {
    let (service, socket) = LspService::new(|client| Backend::new(client));
    Server::new(tokio::io::stdin(), tokio::io::stdout(), socket).serve(service).await;

    Ok(())
}
