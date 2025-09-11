
#[derive(Debug)]
pub struct RestRequest {
  node_list: Vec<String>,
  has_submitted_hash_table: bool,
  is_sent_ip: bool,
}

impl RestRequest {
  #[tokio::main]
  pub async fn get_nodes(self) -> Result<(), reqwest::Error> {
      let url = "https://register-node.dulovar.com/";
      let response = reqwest::get(url).await?;
      let body = response.text().await?;
      println!("Response body: {}", body);
      Ok(())
  }
}

pub struct RestRequestBuilder {
    node_list: Vec<String>,
    is_sent_ip: bool,
    has_submitted_hash_table: bool,
}

impl RestRequestBuilder {
    pub fn new() -> Self {
        Self {
            node_list: Vec::new(),
            has_submitted_hash_table: false,
            is_sent_ip: false,
        }
    }

    pub fn node_list(mut self, list: Vec<String>) -> Self {
        self.node_list = list;
        self
    }

    pub fn has_submitted_hash_table(mut self, value: bool) -> Self {
        self.has_submitted_hash_table = value;
        self
    }

    pub fn is_sent_ip(mut self, value: bool) -> Self {
        self.is_sent_ip = value;
        self
    }

    pub fn build(self) -> RestRequest {
        RestRequest {
            node_list: self.node_list,
            is_sent_ip: self.is_sent_ip,
            has_submitted_hash_table: self.has_submitted_hash_table,
        }
    }
}
