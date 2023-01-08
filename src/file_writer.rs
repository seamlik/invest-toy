use std::path::Path;

#[derive(Default)]
pub struct FileWriter;

#[mockall::automock]
impl FileWriter {
    pub async fn write(&self, path: &Path, content: &[u8]) -> std::io::Result<()> {
        tokio::fs::write(path, content).await
    }
}
