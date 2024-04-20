use rusty_s3::S3Action;

/// Simple abstraction around object storages
pub enum ObjectStore {
    S3 {
        credentials: rusty_s3::Credentials,
        bucket: rusty_s3::Bucket
    },
    Local {
        prefix: String,
    },
}

impl ObjectStore {
    /// Note that duration is only supported for S3
    /// 
    /// On S3, this returns a presigned URL, on local, it returns a file:// url
    pub fn get_url(&self, key: &str, duration: std::time::Duration) -> String {
        match self {
            ObjectStore::S3 { credentials, bucket } => {
                let action = bucket.get_object(Some(credentials), key);
                let url = action.sign(duration);
                url.to_string()
            }
            ObjectStore::Local { prefix } => {
                format!("file://{}/{}", prefix, key)
            }
        }
    }

    pub async fn delete(&self, client: &reqwest::Client, key: &str) -> Result<(), crate::Error> {
        match self {
            ObjectStore::S3 { credentials, bucket } => {
                let mut action = bucket.delete_object(Some(credentials), key);
                action
                .query_mut()
                .insert("response-cache-control", "no-cache, no-store");
                
                let url = action.sign(std::time::Duration::from_secs(30));
                let response = client
                    .delete(url)
                    .send()
                    .await
                    .map_err(|e| format!("Failed to delete object: {}", e))?;

                if !response.status().is_success() {
                    let text = response.text().await.map_err(|e| format!("Failed to read response: {}", e))?;
                    return Err(format!("Failed to delete object: {}", text).into());
                }

                Ok(())
            }
            ObjectStore::Local { prefix } => {
                let path = std::path::Path::new(prefix).join(key);
                std::fs::remove_file(path).map_err(|e| format!("Failed to delete object: {}", e))?;

                Ok(())
            }
        }
    }
}
