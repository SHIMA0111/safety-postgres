use std::error::Error;
use safety_postgres::generator::base::Parameters;
use safety_postgres::Variable;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let parameter1 = Parameters::from(vec![Variable::Text("Hello".to_string())]);
    let parameter2 = Parameters::from(vec![Variable::Text("World".to_string())]);
    println!("Param1 + Param2: {}",
             parameter1 + parameter2);

    Ok(())
}

