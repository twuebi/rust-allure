use std::path::PathBuf;
use uuid::Uuid;

pub(crate) async fn write_attachment(
    mime: crate::reporter::Mime,
    content: &[u8],
    mut allure_dir: PathBuf,
) -> anyhow::Result<PathBuf> {
    let of = format!("{}-attachment.{}", Uuid::now_v7(), mime.as_ext()).into();
    allure_dir.push(&of);

    tokio::fs::write(allure_dir, &content).await?;
    Ok(of)
}
