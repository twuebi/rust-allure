use crate::reporter::Mime;
use std::path::PathBuf;
use uuid::Uuid;

pub(crate) async fn write_attachment(
    mime: Mime,
    content: &[u8],
    mut allure_dir: PathBuf,
) -> anyhow::Result<PathBuf> {
    let of = format!("{}-attachment.{}", Uuid::new_v4(), mime.as_ext()).into();
    allure_dir.push(&of);

    tokio::fs::write(allure_dir, &content).await?;
    Ok(of)
}
