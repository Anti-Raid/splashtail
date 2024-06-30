use rusty_s3::S3Action;

const CHUNK_SIZE: usize = 5 * 1024 * 1024;
const MULTIPART_MIN_SIZE: usize = 50 * 1024 * 1024;
const MULTIPART_SIGN_DURATION: std::time::Duration = std::time::Duration::from_secs(30);
const PUT_OBJECT_TIME: std::time::Duration = std::time::Duration::from_secs(30);

/// Simple abstraction around object storages
pub enum ObjectStore {
    S3 {
        credentials: rusty_s3::Credentials,
        bucket: rusty_s3::Bucket,
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
            ObjectStore::S3 {
                credentials,
                bucket,
            } => {
                let action = bucket.get_object(Some(credentials), key);
                let url = action.sign(duration);
                url.to_string()
            }
            ObjectStore::Local { prefix } => {
                format!("file://{}/{}", prefix, key)
            }
        }
    }

    /// Uploads a file to the object store with a given key
    pub async fn upload_file(
        &self,
        client: &reqwest::Client,
        key: &str,
        data: &[u8],
    ) -> Result<(), crate::Error> {
        match self {
            ObjectStore::S3 {
                credentials,
                bucket,
            } => {
                if data.len() > MULTIPART_MIN_SIZE {
                    let action = bucket.create_multipart_upload(Some(credentials), key);
                    let url = action.sign(MULTIPART_SIGN_DURATION);
                    let response = client
                        .post(url)
                        .send()
                        .await
                        .map_err(|e| format!("Failed to create object: {}", e))?;

                    if !response.status().is_success() {
                        let text = response
                            .text()
                            .await
                            .map_err(|e| format!("Failed to read response: {}", e))?;
                        return Err(format!("Failed to create object: {}", text).into());
                    }

                    let body = response
                        .text()
                        .await
                        .map_err(|e| format!("Failed to read response: {}", e))?;

                    let multipart =
                        rusty_s3::actions::CreateMultipartUpload::parse_response(&body)?;

                    // Upload parts
                    let mut error: Option<crate::Error> = None;
                    let mut parts = vec![];
                    loop {
                        let action =
                            bucket.upload_part(Some(credentials), key, 1, multipart.upload_id());
                        let url = action.sign(MULTIPART_SIGN_DURATION);

                        // Split into 5 mb parts
                        let range = std::ops::Range {
                            start: parts.len() * CHUNK_SIZE,
                            end: std::cmp::min(data.len(), (parts.len() + 1) * CHUNK_SIZE),
                        };

                        let send_data = &data[range.start..range.end];

                        let etag = {
                            let response = match client
                                .put(url)
                                .body(send_data.to_vec())
                                .send()
                                .await
                                .map_err(|e| format!("Failed to create object: {}", e))
                            {
                                Ok(response) => response,
                                Err(e) => {
                                    error = Some(e.into());
                                    break;
                                }
                            };

                            if !response.status().is_success() {
                                let text = match response
                                    .text()
                                    .await
                                    .map_err(|e| format!("Failed to read response: {}", e))
                                {
                                    Ok(text) => text,
                                    Err(e) => {
                                        error = Some(e.into());
                                        break;
                                    }
                                };

                                error = Some(format!("Failed to create object: {}", text).into());
                                break;
                            }

                            let etag_header =
                                match response.headers().get("ETag").ok_or("Missing ETag header") {
                                    Ok(etag) => etag,
                                    Err(e) => {
                                        error = Some(e.into());
                                        break;
                                    }
                                };

                            let etag_str = match etag_header.to_str() {
                                Ok(etag_str) => etag_str,
                                Err(e) => {
                                    error = Some(e.into());
                                    break;
                                }
                            };

                            etag_str.to_string()
                        };

                        parts.push(etag);

                        if range.end == data.len() {
                            break;
                        }
                    }

                    if let Some(error) = error {
                        // Abort upload on error
                        let action = bucket.abort_multipart_upload(
                            Some(credentials),
                            key,
                            multipart.upload_id(),
                        );

                        let url = action.sign(std::time::Duration::from_secs(30));

                        client
                            .delete(url)
                            .send()
                            .await
                            .map_err(|e| format!("Failed to abort upload: {}", e))?;

                        return Err(error);
                    }

                    // Complete upload
                    let mut parts_str: Vec<&str> = vec![];

                    for part in &parts {
                        parts_str.push(part)
                    }

                    let action = bucket.complete_multipart_upload(
                        Some(credentials),
                        key,
                        multipart.upload_id(),
                        parts_str.into_iter(),
                    );

                    let url = action.sign(MULTIPART_SIGN_DURATION);

                    let response = client
                        .post(url)
                        .body(action.body())
                        .send()
                        .await
                        .map_err(|e| format!("Failed to complete upload: {}", e))?;

                    if !response.status().is_success() {
                        let text = response
                            .text()
                            .await
                            .map_err(|e| format!("Failed to read response: {}", e))?;
                        return Err(format!("Failed to complete upload: {}", text).into());
                    }

                    Ok(())
                } else {
                    let action = bucket.put_object(Some(credentials), key);
                    let url = action.sign(PUT_OBJECT_TIME);
                    let response = client
                        .put(url)
                        .body(data.to_vec())
                        .send()
                        .await
                        .map_err(|e| format!("Failed to create object: {}", e))?;

                    if !response.status().is_success() {
                        let text = response
                            .text()
                            .await
                            .map_err(|e| format!("Failed to read response: {}", e))?;
                        return Err(format!("Failed to create object: {}", text).into());
                    }

                    Ok(())
                }
            }
            ObjectStore::Local { prefix } => {
                let path = std::path::Path::new(prefix).join(key);
                std::fs::create_dir_all(path.parent().unwrap())
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
                std::fs::write(path, data).map_err(|e| format!("Failed to write object: {}", e))?;

                Ok(())
            }
        }
    }

    pub async fn delete(&self, client: &reqwest::Client, key: &str) -> Result<(), crate::Error> {
        match self {
            ObjectStore::S3 {
                credentials,
                bucket,
            } => {
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
                    let text = response
                        .text()
                        .await
                        .map_err(|e| format!("Failed to read response: {}", e))?;
                    return Err(format!("Failed to delete object: {}", text).into());
                }

                Ok(())
            }
            ObjectStore::Local { prefix } => {
                let path = std::path::Path::new(prefix).join(key);
                std::fs::remove_file(path)
                    .map_err(|e| format!("Failed to delete object: {}", e))?;

                Ok(())
            }
        }
    }
}
