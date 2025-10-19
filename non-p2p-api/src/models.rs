use serde::{Deserialize, Serialize};
use worker::{Request, Response, RouteContext};

/// Input structure for creating nodes via API
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct NodeInput {
    pub address: String,
    #[serde(default)]
    pub valid: i8,
    #[serde(default)]
    pub master: i8,
}

impl NodeInput {
    /// Creates a Node from an input Node, ensuring ID is None for database insertion
    pub fn from_input(input: NodeInput) -> Self {
        Self {
            address: input.address,
            valid: input.valid,
            master: input.master,
        }
    }
}

// Database operations
const INSERT_NODE_QUERY: &str =
    "INSERT INTO nodes (address, valid, master) VALUES (?, false, false)";
const SELECT_NODES_QUERY: &str = "SELECT * FROM nodes";

/// Creates a new node in the database
pub async fn create_node(
    mut req: Request,
    ctx: RouteContext<()>,
) -> Result<Response, worker::Error> {
    let input_node: NodeInput = req.json().await?;
    let d1 = ctx.env.d1("DB")?;

    let node = NodeInput::from_input(input_node);

    let statement = d1.prepare(INSERT_NODE_QUERY);
    let query = statement.bind(&[node.address.into()])?;

    query.run().await?;

    Response::ok("Node created successfully")
}

/// Retrieves all nodes from the database
pub async fn select_nodes(ctx: RouteContext<()>) -> Result<Response, worker::Error> {
    let d1 = ctx.env.d1("DB")?;

    let statement = d1.prepare(SELECT_NODES_QUERY);
    let results = statement.all().await?;
    let nodes: Vec<NodeInput> = results.results()?;

    Response::from_json(&nodes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_from_input() {
        let input = NodeInput {
            address: "test.server.com:8000".to_string(),
            valid: 1,
            master: 0,
        };

        let db_node = NodeInput::from_input(input.clone());

        assert_eq!(db_node.address, input.address);
        assert_eq!(db_node.valid, input.valid);
        assert_eq!(db_node.master, input.master);
    }

    #[test]
    fn test_node_input_with_missing_fields() {
        let json = r#"{"address":"test.server.com:8000"}"#;
        let input: NodeInput = serde_json::from_str(json).unwrap();

        assert_eq!(input.address, "test.server.com:8000");
        assert_eq!(input.valid, 0);
        assert_eq!(input.master, 0);
    }

    #[test]
    fn test_node_input_with_all_fields() {
        let json = r#"{"address":"test.server.com:8000","valid":1,"master":0}"#;
        let input: NodeInput = serde_json::from_str(json).unwrap();

        assert_eq!(input.address, "test.server.com:8000");
        assert_eq!(input.valid, 1);
        assert_eq!(input.master, 0);
    }
}
