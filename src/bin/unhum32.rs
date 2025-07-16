use std::io::{Read, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    std::io::stdin().read_to_end(&mut buffer)?;
    let s = String::from_utf8(buffer)?;
    let output = hum32::decode(&s, false)
        .map_err(|e| std::io::Error::other(format!("{:?}", e)))?;
    std::io::stdout().write_all(&output)?;
    Ok(())
}
