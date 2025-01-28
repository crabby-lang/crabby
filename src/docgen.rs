use crate::parser::ast::Statement;

pub struct Documentation {
    pub name: String,
    pub description: String,
    pub params: Vec<(String, String)>,
    pub return_type: String,
}

pub fn generate_docs(statement: &Vec<Statement>) -> Vec<Documentation> {
    let mut docs = Vec::new();

    for node in statement {
        match node {
            Statement::FunctionDef { name, params, return_type, docstring } => {
                docs.push(Documentation {
                    name: name.clone(),
                    description: docstring.clone().unwrap_or_else(|| "No description provided.".to_string()),
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
        "markdown" => export_to_markdown(docs),
        "doublequotes" => export_to_double_quotes(docs),
        _ => eprintln!("Unsupported format!"),
    }
}

fn export_to_markdown(docs: Vec<Documentation>) {
    for doc in docs {
        println!("### {}", doc.name);
        println!("\n{}", doc.description);
        println!("\n### Parameters:");
        for (name, typ) in doc.params {
            println!("- {}: {}", name, typ);
        }
        println!("\n### Returns: {}", doc.return_type);
        println!("\n---\n")
    }
}

fn export_to_double_quotes(docs: Vec<Documentation>) {
    for doc in docs {
        println!("\"\"\"");
        println!("function: {}", doc.name);
        println!("\n{}", doc.description);
        println!("\nParameters:");
        for (name, typ) in doc.params {
            println!("    {}: {}", name, typ);
        }
        println!("\nReturns: {}", doc.return_type);
        println!("\"\"\"");
        println!();
    }
}
