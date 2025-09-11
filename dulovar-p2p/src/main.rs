mod rest_requests;
use rest_requests::RestRequestBuilder;
fn main() {
    let r = RestRequestBuilder::new()
        .node_list(vec![1, 2, 3])
        .is_sent_ip(true)
        .build();
    
    match r.get_nodes() {
        Ok(response) => println!("Response: {:#?}", response),
        Err(e) => println!("Error: {:?}", e),
    }
}

