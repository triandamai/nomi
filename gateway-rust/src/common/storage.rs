use dotenvy::var;
use s3::creds::Credentials;
use s3::request::ResponseData;
use s3::{Bucket, Region};
use serde::{Deserialize, Serialize};
use tracing::log::info;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExploreItem {
    pub r#type: String, // "folder", "file", "bucket"
    pub name: String,
    pub full_path: String,
}

#[derive(Clone)]
pub struct StorageClient {
    access_key: String,
    secret_key: String,
    url_server: String,
}

impl StorageClient {
    pub async fn explore_storage(
        &self,
        prefix: Option<String>,
    ) -> Result<Vec<ExploreItem>, String> {
        let credentials = Credentials::new(
            Some(self.access_key.clone().as_str()),
            Some(self.secret_key.clone().as_str()),
            None,
            None,
            None,
        );
        if credentials.is_err() {
            return Err(credentials.unwrap_err().to_string());
        }
        let credentials = credentials.unwrap();

        let region = Region::Custom {
            region: "asia".to_string(),
            endpoint: self.url_server.clone(),
        };

        if prefix.is_none() || prefix.as_deref().unwrap().is_empty() {
            let res = Bucket::list_buckets(region, credentials).await;
            if res.is_err() {
                return Err(res.unwrap_err().to_string());
            }
            let res = res.unwrap();

            let mut items = Vec::new();
            for name in res.bucket_names() {
                items.push(ExploreItem {
                    r#type: "bucket".to_string(),
                    name: name.clone(),
                    full_path: name.clone(),
                });
            }
            return Ok(items);
        }

        let prefix_str = prefix.unwrap();
        let mut parts = prefix_str.splitn(2, '/');
        let bucket_name = parts.next().unwrap();
        let obj_prefix = parts.next().unwrap_or("").to_string();

        let bucket = Bucket::new(bucket_name, region, credentials);
        if bucket.is_err() {
            return Err(bucket.unwrap_err().to_string());
        }
        let bucket = bucket.unwrap().with_path_style();

        let res = bucket.list(obj_prefix, Some("/".to_string())).await;
        if res.is_err() {
            return Err(res.unwrap_err().to_string());
        }
        let res = res.unwrap();

        let mut items = Vec::new();
        for r in res {
            if let Some(prefixes) = r.common_prefixes {
                for cp in prefixes {
                    let name = cp
                        .prefix
                        .trim_end_matches('/')
                        .split('/')
                        .last()
                        .unwrap_or("")
                        .to_string();
                    items.push(ExploreItem {
                        r#type: "folder".to_string(),
                        name,
                        full_path: format!("{}/{}", bucket_name, cp.prefix),
                    });
                }
            }
            for obj in r.contents {
                if obj.key.ends_with('/') {
                    continue;
                }
                let name = obj.key.split('/').last().unwrap_or("").to_string();
                items.push(ExploreItem {
                    r#type: "file".to_string(),
                    name,
                    full_path: format!("{}/{}", bucket_name, obj.key),
                });
            }
        }

        Ok(items)
    }

    pub fn new(access_key: String, secret_key: String, url_server: String) -> StorageClient {
        info!(target:"app::minio","Init minio");
        StorageClient {
            access_key,
            secret_key,
            url_server,
        }
    }

    pub async fn get_file(
        &self,
        bucket_name: String,
        file_name: String,
    ) -> Result<ResponseData, String> {
        info!(target:"app::minio","getting file bucket:{} filename:{} ",bucket_name, file_name);
        let credentials = Credentials::new(
            Some(self.access_key.clone().as_str()),
            Some(self.secret_key.clone().as_str()),
            None,
            None,
            None,
        );
        if credentials.is_err() {
            let err = credentials.unwrap_err();
            info!(target:"app::minio","Credentials error:{}",&err);
            return Err(err.to_string());
        }
        let credentials = credentials.unwrap();
        let bucket = Bucket::new(
            bucket_name.as_str(),
            Region::Custom {
                region: "asia".to_string(),
                endpoint: self.url_server.clone(),
            },
            credentials,
        );
        if bucket.is_err() {
            let err = bucket.unwrap_err();
            info!(target:"app::minio","Bucket error :{}",&err);
            return Err(err.to_string());
        }
        let filename = format!("/{}", file_name);
        let bucket = bucket.unwrap().with_path_style();

        let file = bucket.get_object(filename.clone()).await;
        if file.is_err() {
            let err = file.unwrap_err();
            info!(target:"app::minio","Bucket error :{}",&err);
            return Err(err.to_string());
        }

        {
            info!(target: "get_object","from minio {}", &filename);
        }
        if file.is_err() {
            let err = file.unwrap_err();
            info!(target:"app::minio","Bucket error :{}",&err);
            return Err(err.to_string());
        }
        let file = file.unwrap();
        Ok(file)
    }

    pub async fn upload_byte(
        &self,
        bucket_name: String,
        path: String,
        data: Vec<u8>,
    ) -> Result<String, String> {
        let credentials = Credentials::new(
            Some(self.access_key.clone().as_str()),
            Some(self.secret_key.clone().as_str()),
            None,
            None,
            None,
        );
        if credentials.is_err() {
            return Err(credentials.unwrap_err().to_string());
        }
        let credentials = credentials.unwrap();
        let bucket = Bucket::new(
            bucket_name.as_str(),
            Region::Custom {
                region: "asia".to_string(),
                endpoint: self.url_server.clone(),
            },
            credentials,
        );
        if bucket.is_err() {
            return Err(bucket.unwrap_err().to_string());
        }
        let bucket = bucket.unwrap().with_path_style();

        let upload = bucket.put_object(path.as_str(), &data).await;
        match upload {
            Ok(_) => Ok(path),
            Err(e) => Err(format!("Error uploading file: {}", e)),
        }
    }

    pub async fn upload_file(
        &self,
        file_temp_location: String,
        bucket_name: String,
        file_path: String,
    ) -> Result<String, String> {
        let credentials = Credentials::new(
            Some(self.access_key.clone().as_str()),
            Some(self.secret_key.clone().as_str()),
            None,
            None,
            None,
        );
        if credentials.is_err() {
            return Err(credentials.unwrap_err().to_string());
        }
        let credentials = credentials.unwrap();
        let bucket = Bucket::new(
            bucket_name.as_str(),
            Region::Custom {
                region: "asia".to_string(),
                endpoint: self.url_server.clone(),
            },
            credentials,
        );
        if bucket.is_err() {
            return Err(bucket.unwrap_err().to_string());
        }
        let bucket = bucket.unwrap().with_path_style();

        let file = tokio::fs::read(file_temp_location).await;
        if file.is_err() {
            return Err(file.unwrap_err().to_string());
        }
        let file = file.unwrap();
        let upload = bucket.put_object(file_path.as_str(), &file).await;
        match upload {
            Ok(_) => Ok("Successfully uploaded file".to_string()),
            Err(e) => Err(format!("Error uploading file: {}", e)),
        }
    }

    pub async fn delete_file(
        &self,
        file_path: String,
        bucket_name: String,
    ) -> Result<String, String> {
        let credentials = Credentials::new(
            Some(self.access_key.clone().as_str()),
            Some(self.secret_key.clone().as_str()),
            None,
            None,
            None,
        );
        if credentials.is_err() {
            return Err(credentials.unwrap_err().to_string());
        }
        let credentials = credentials.unwrap();
        let bucket = Bucket::new(
            bucket_name.as_str(),
            Region::Custom {
                region: "asia".to_string(),
                endpoint: self.url_server.clone(),
            },
            credentials,
        );
        if bucket.is_err() {
            return Err(bucket.unwrap_err().to_string());
        }
        let bucket = bucket.unwrap().with_path_style();

        let upload = bucket.delete_object(file_path.as_str()).await;
        match upload {
            Ok(_) => Ok("Successfully delete  file".to_string()),
            Err(e) => Err(format!("Error uploading file: {}", e)),
        }
    }

    pub fn get_full_url(&self, path: &str) -> String {
        let base_url = var("PUBLIC_GATEWAY_URL").expect("http://localhost:8000/api");
        format!("{}/files/{}", base_url, path)
    }
}
