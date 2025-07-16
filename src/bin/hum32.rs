use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    std::io::stdin().read_to_end(&mut buffer)?;
    let output = hum32::encode(&buffer, None)
        .map_err(|e| std::io::Error::other(format!("{:?}", e)))?;
    println!("{output}");
    Ok(())
}
