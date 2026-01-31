use anyhow::Result;
use std::fs;
use std::path::Path;
use tree_sitter::Parser;
use crate::authorship::{AuthorshipAnalyzer, AuthorshipInfo};

/// THE SHREDDER: Extracts capabilities from code using AST parsing
pub struct Shredder {
    parser: Parser,
    authorship_analyzer: Option<AuthorshipAnalyzer>,
}

#[derive(Debug, Clone)]
pub struct Capability {
    pub name: String,
    pub kind: CapabilityKind,
    pub line: usize,
    pub code_snippet: String, // The actual code for embedding
    pub authorship: Option<AuthorshipInfo>, // Git blame authorship info
}

#[derive(Debug, Clone)]
pub enum CapabilityKind {
    Function,
    Class,
    ApiRoute,
    Component, // React/Vue components
}

impl Shredder {
    pub fn new() -> Result<Self> {
        let parser = Parser::new();
        Ok(Self { 
            parser,
            authorship_analyzer: None,
        })
    }

    /// Create a new Shredder with authorship tracking enabled
    pub fn with_authorship(
        repo_path: &Path,
        user_email: Option<String>,
        user_name: Option<String>,
    ) -> Result<Self> {
        let parser = Parser::new();
        let authorship_analyzer = AuthorshipAnalyzer::new(repo_path, user_email, user_name).ok();
        Ok(Self {
            parser,
            authorship_analyzer,
        })
    }

    /// Extract code snippet from a node (limited to 500 chars for embedding)
    fn extract_code_snippet(node: &tree_sitter::Node, source: &str) -> String {
        let start_byte = node.start_byte();
        let end_byte = node.end_byte();
        let snippet = &source[start_byte..end_byte];
        
        // Limit to 500 chars, but try to keep it readable
        if snippet.len() > 500 {
            snippet.chars().take(500).collect::<String>() + "..."
        } else {
            snippet.to_string()
        }
    }

    /// Shred a file and extract its capabilities
    pub fn shred_file(&mut self, path: &Path) -> Result<Vec<Capability>> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let language = match ext {
            "ts" | "tsx" => tree_sitter_typescript::language_typescript(),
            "js" | "jsx" => tree_sitter_typescript::language_typescript(), // JS uses TS parser
            "rs" => tree_sitter_rust::language(),
            "py" => tree_sitter_python::language(),
            "go" => tree_sitter_go::language(),
            _ => return Ok(vec![]), // Unsupported language
        };

        self.parser.set_language(language)?;

        let source_code = fs::read_to_string(path)?;
        let tree = self.parser.parse(&source_code, None).ok_or_else(|| {
            anyhow::anyhow!("Failed to parse {}", path.display())
        })?;

        let root_node = tree.root_node();
        let mut capabilities = Vec::new();

        // Extract based on language
        match ext {
            "ts" | "tsx" | "js" | "jsx" => {
                capabilities.extend(self.extract_typescript_capabilities(&root_node, &source_code, path)?);
            }
            "rs" => {
                capabilities.extend(self.extract_rust_capabilities(&root_node, &source_code, path)?);
            }
            "py" => {
                capabilities.extend(self.extract_python_capabilities(&root_node, &source_code, path)?);
            }
            "go" => {
                capabilities.extend(self.extract_go_capabilities(&root_node, &source_code, path)?);
            }
            _ => {}
        }

        Ok(capabilities)
    }

    /// Extract TypeScript/JavaScript capabilities
    fn extract_typescript_capabilities(
        &self,
        root: &tree_sitter::Node,
        source: &str,
        path: &Path,
    ) -> Result<Vec<Capability>> {
        let mut capabilities = Vec::new();
        self.traverse_typescript_node(root, source, &mut capabilities, false, path)?;
        Ok(capabilities)
    }

    fn traverse_typescript_node(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        capabilities: &mut Vec<Capability>,
        is_exported: bool,
        path: &Path,
    ) -> Result<()> {
        let node_type = node.kind();
        let mut current_is_exported = is_exported;

        // Check for export keyword
        if node_type == "export_statement" || node_type == "export" {
            current_is_exported = true;
        }

        match node_type {
            "function_declaration" => {
                // Look for function name
                for i in 0..node.child_count() {
                    let child = node.child(i).unwrap();
                    if child.kind() == "identifier" {
                        let name = child.utf8_text(source.as_bytes())?.to_string();
                        if !name.is_empty() {
                            // Check if it's a React component (PascalCase)
                            let is_component = name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
                            
                            let code_snippet = Self::extract_code_snippet(node, source);
                            let line = node.start_position().row + 1;
                            let authorship = self.get_authorship(path, line, line);
                            
                            capabilities.push(Capability {
                                name: name.clone(),
                                kind: if is_component {
                                    CapabilityKind::Component
                                } else {
                                    CapabilityKind::Function
                                },
                                line,
                                code_snippet,
                                authorship,
                            });
                        }
                        break;
                    }
                }
            }
            "class_declaration" => {
                // Look for class name
                for i in 0..node.child_count() {
                    let child = node.child(i).unwrap();
                    if child.kind() == "type_identifier" || child.kind() == "identifier" {
                        let name = child.utf8_text(source.as_bytes())?.to_string();
                        if !name.is_empty() {
                            let code_snippet = Self::extract_code_snippet(node, source);
                            let line = node.start_position().row + 1;
                            let authorship = self.get_authorship(path, line, line);
                            capabilities.push(Capability {
                                name,
                                kind: CapabilityKind::Class,
                                line,
                                code_snippet,
                                authorship,
                            });
                        }
                        break;
                    }
                }
            }
            "variable_declarator" => {
                // Check for arrow functions or const exports
                let mut name = None;
                let mut has_function = false;
                for i in 0..node.child_count() {
                    let child = node.child(i).unwrap();
                    if child.kind() == "identifier" {
                        name = Some(child.utf8_text(source.as_bytes())?.to_string());
                    }
                    if child.kind() == "arrow_function" || child.kind() == "function" {
                        has_function = true;
                    }
                }
                if has_function {
                    if let Some(n) = name {
                        let is_component = n.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
                        let code_snippet = Self::extract_code_snippet(node, source);
                        let line = node.start_position().row + 1;
                        let authorship = self.get_authorship(path, line, line);
                        
                        capabilities.push(Capability {
                            name: n,
                            kind: if is_component {
                                CapabilityKind::Component
                            } else {
                                CapabilityKind::Function
                            },
                            line,
                            code_snippet,
                            authorship,
                        });
                    }
                }
            }
            "method_definition" => {
                // Extract method names
                for i in 0..node.child_count() {
                    let child = node.child(i).unwrap();
                    if child.kind() == "property_identifier" || child.kind() == "identifier" {
                        let name = child.utf8_text(source.as_bytes())?.to_string();
                        if !name.is_empty() && (name == "get" || name == "post" || name == "put" || name == "delete") {
                            // This might be an API route
                            let code_snippet = Self::extract_code_snippet(node, source);
                            let line = node.start_position().row + 1;
                            let authorship = self.get_authorship(path, line, line);
                            
                            capabilities.push(Capability {
                                name,
                                kind: CapabilityKind::ApiRoute,
                                line,
                                code_snippet,
                                authorship,
                            });
                        }
                        break;
                    }
                }
            }
            _ => {}
        }

        // Recurse into children
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.traverse_typescript_node(&child, source, capabilities, current_is_exported, path)?;
            }
        }

        Ok(())
    }

    /// Extract Rust capabilities
    fn extract_rust_capabilities(
        &self,
        root: &tree_sitter::Node,
        source: &str,
        path: &Path,
    ) -> Result<Vec<Capability>> {
        let mut capabilities = Vec::new();
        self.traverse_rust_node(root, source, &mut capabilities, false, path)?;
        Ok(capabilities)
    }

    fn traverse_rust_node(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        capabilities: &mut Vec<Capability>,
        is_pub: bool,
        path: &Path,
    ) -> Result<()> {
        let node_type = node.kind();
        let mut current_is_pub = is_pub;

        // Check for pub keyword
        if node_type == "visibility_modifier" {
            current_is_pub = true;
        }

        match node_type {
            "function_item" => {
                let mut name = None;
                let mut found_pub = current_is_pub;

                // Check parent for pub
                if let Some(parent) = node.parent() {
                    for i in 0..parent.child_count() {
                        let child = parent.child(i).unwrap();
                        if child.kind() == "visibility_modifier" {
                            found_pub = true;
                        }
                    }
                }

                // Extract function name
                for i in 0..node.child_count() {
                    let child = node.child(i).unwrap();
                    if child.kind() == "identifier" {
                        name = Some(child.utf8_text(source.as_bytes())?.to_string());
                        break;
                    }
                }

                if found_pub && name.is_some() {
                    let code_snippet = Self::extract_code_snippet(node, source);
                    let line = node.start_position().row + 1;
                    let authorship = self.get_authorship(path, line, line);
                    
                    capabilities.push(Capability {
                        name: name.unwrap(),
                        kind: CapabilityKind::Function,
                        line,
                        code_snippet,
                        authorship,
                    });
                }
            }
            "struct_item" | "impl_item" => {
                // Extract struct/impl names
                for i in 0..node.child_count() {
                    let child = node.child(i).unwrap();
                    if child.kind() == "type_identifier" {
                        let name = child.utf8_text(source.as_bytes())?.to_string();
                        let code_snippet = Self::extract_code_snippet(node, source);
                        let line = node.start_position().row + 1;
                        let authorship = self.get_authorship(path, line, line);
                        capabilities.push(Capability {
                            name,
                            kind: CapabilityKind::Class,
                            line,
                            code_snippet,
                            authorship,
                        });
                        break;
                    }
                }
            }
            _ => {}
        }

        // Recurse into children
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.traverse_rust_node(&child, source, capabilities, current_is_pub, path)?;
            }
        }

        Ok(())
    }

    /// Extract Python capabilities
    fn extract_python_capabilities(
        &self,
        root: &tree_sitter::Node,
        source: &str,
        path: &Path,
    ) -> Result<Vec<Capability>> {
        let mut capabilities = Vec::new();
        self.traverse_python_node(root, source, &mut capabilities, path)?;
        Ok(capabilities)
    }

    fn traverse_python_node(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        capabilities: &mut Vec<Capability>,
        path: &Path,
    ) -> Result<()> {
        let node_type = node.kind();

        match node_type {
            "function_definition" => {
                // Extract function name
                for i in 0..node.child_count() {
                    let child = node.child(i).unwrap();
                    if child.kind() == "identifier" {
                        let name = child.utf8_text(source.as_bytes())?.to_string();
                        let code_snippet = Self::extract_code_snippet(node, source);
                        let line = node.start_position().row + 1;
                        let authorship = self.get_authorship(path, line, line);
                        
                        capabilities.push(Capability {
                            name,
                            kind: CapabilityKind::Function,
                            line,
                            code_snippet,
                            authorship,
                        });
                        break;
                    }
                }
            }
            "class_definition" => {
                // Extract class name
                for i in 0..node.child_count() {
                    let child = node.child(i).unwrap();
                    if child.kind() == "identifier" {
                        let name = child.utf8_text(source.as_bytes())?.to_string();
                        let code_snippet = Self::extract_code_snippet(node, source);
                        let line = node.start_position().row + 1;
                        let authorship = self.get_authorship(path, line, line);
                        capabilities.push(Capability {
                            name,
                            kind: CapabilityKind::Class,
                            line,
                            code_snippet,
                            authorship,
                        });
                        break;
                    }
                }
            }
            _ => {}
        }

        // Recurse
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.traverse_python_node(&child, source, capabilities, path)?;
            }
        }

        Ok(())
    }

    /// Extract Go capabilities
    fn extract_go_capabilities(
        &self,
        root: &tree_sitter::Node,
        source: &str,
        path: &Path,
    ) -> Result<Vec<Capability>> {
        let mut capabilities = Vec::new();
        self.traverse_go_node(root, source, &mut capabilities, path)?;
        Ok(capabilities)
    }

    fn traverse_go_node(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        capabilities: &mut Vec<Capability>,
        path: &Path,
    ) -> Result<()> {
        let node_type = node.kind();

        match node_type {
            "method_declaration" | "function_declaration" => {
                // Extract function name
                for i in 0..node.child_count() {
                    let child = node.child(i).unwrap();
                    if child.kind() == "identifier" {
                        let name = child.utf8_text(source.as_bytes())?.to_string();
                        // Only export if it starts with uppercase (Go exports)
                        if name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                            let code_snippet = Self::extract_code_snippet(node, source);
                            let line = node.start_position().row + 1;
                            let authorship = self.get_authorship(path, line, line);
                            capabilities.push(Capability {
                                name,
                                kind: CapabilityKind::Function,
                                line,
                                code_snippet,
                                authorship,
                            });
                        }
                        break;
                    }
                }
            }
            "type_declaration" => {
                // Extract type/struct names
                for i in 0..node.child_count() {
                    let child = node.child(i).unwrap();
                    if child.kind() == "type_identifier" {
                        let name = child.utf8_text(source.as_bytes())?.to_string();
                        let code_snippet = Self::extract_code_snippet(node, source);
                        let line = node.start_position().row + 1;
                        let authorship = self.get_authorship(path, line, line);
                        capabilities.push(Capability {
                            name,
                            kind: CapabilityKind::Class,
                            line,
                            code_snippet,
                            authorship,
                        });
                        break;
                    }
                }
            }
            _ => {}
        }

        // Recurse
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.traverse_go_node(&child, source, capabilities, path)?;
            }
        }

        Ok(())
    }

    /// Get authorship info for a line range in a file
    fn get_authorship(&self, path: &Path, start_line: usize, end_line: usize) -> Option<AuthorshipInfo> {
        if let Some(ref analyzer) = self.authorship_analyzer {
            analyzer.analyze_file(path, start_line, end_line).ok()
        } else {
            None
        }
    }
}
