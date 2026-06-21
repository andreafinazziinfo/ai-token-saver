use anyhow::Result;

pub fn list() -> Result<()> {
    let list = rtk_db::artifact::artifact_list()?;
    if list.is_empty() {
        println!("No artifacts stored.");
        return Ok(());
    }
    
    println!("{:<24} | {:<12} | {:<20} | {}", "ID", "Type", "Created At", "Metadata");
    println!("{}", "-".repeat(80));
    for art in list {
        let meta = art.metadata_json.as_deref().unwrap_or("{}");
        println!("{:<24} | {:<12} | {:<20} | {}", art.id, art.r#type, art.created_at, meta);
    }
    Ok(())
}

pub fn get(id: &str) -> Result<()> {
    let art = rtk_db::artifact::artifact_get(id)?;
    print!("{}", art.content);
    Ok(())
}

pub fn gc() -> Result<()> {
    let deleted = rtk_db::artifact::artifact_gc()?;
    println!("🗑️ Cleaned up {} artifacts older than 30 days.", deleted);
    Ok(())
}
