#[derive(Debug)]
pub enum TardisError {
    /// Error involving accessing the filesystem (e.g. to read
    /// an input executable or write its contents to disk).
    FilesystemError(String),
}
