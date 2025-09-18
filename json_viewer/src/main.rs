fn main() -> anyhow::Result<()> {
    let content = std::fs::read_to_string("business-licences.json")?;
    println!("content: {content}");
    Ok(())
}
