use sha2::Digest;

pub fn sha256(txt: &str) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(txt);
    format!("{:x}", hasher.finalize())
}
