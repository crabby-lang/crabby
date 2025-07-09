use crate::parser::Statement;

pub struct Documentation {
    pub name: String,
    pub description: String,
    pub params: Vec<String>,
    pub body: Box<Statement>,
    pub return_type: String,
}

impl Documentation {
    pub fn generate_docs(statement: &Vec<Statement>) -> Vec<Documentation> {
        let mut docs = Vec::new();

        for node in statement {
            match node {
                Statement::FunctionDef { name, params, body, return_type, docstring } => {
                    docs.push(Documentation {
                        name: name.clone(),
                        body: body.clone(),
                        description: docstring.clone(),
                        params: params.clone(),
                        return_type: return_type.clone(),
                    });
                }
                _ => {}
            }
        }

        docs
    }

    pub fn export_docs(docs: Vec<Documentation>, format: &str) {
        match format {
            "markdown" => Self::export_to_markdown(docs),
            "doublequotes" => Self::export_to_double_quotes(docs),
            _ => println!("Unsupported format"),
        }
    }

    fn export_to_markdown(docs: Vec<Documentation>) {
        for doc in docs {
            println!("# {}", doc.name);
            println!("\n{}\n", doc.description);
            println!("## Parameters");
            for param in doc.params {
                println!("- `{}`", param);
            }
            println!("\n## Returns\n{}\n", doc.return_type);
            println!("---\n");
        }
    }

    fn export_to_double_quotes(docs: Vec<Documentation>) {
        for doc in docs {
            println!("\"{}\"", doc.name);
            println!("\"{}\"", doc.description);
            println!("Parameters:");
            for param in doc.params {
                println!("\"{}\"", param);
            }
            println!("Returns: \"{}\"\n", doc.return_type);
        }
    }
}
