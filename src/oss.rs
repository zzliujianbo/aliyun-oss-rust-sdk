use crate::auth::AuthAPI;
use crate::request::RequestBuilder;
use chrono::{DateTime, Utc};
use reqwest::header::{HeaderMap, InvalidHeaderValue, AUTHORIZATION, CONTENT_TYPE, DATE};

/// OSS配置
#[derive(Debug, Clone)]
pub struct OSS {
    key_id: String,
    key_secret: String,
    endpoint: String,
    bucket: String,
}

unsafe impl Send for OSS {}

unsafe impl Sync for OSS {}

pub trait OSSInfo {
    fn endpoint(&self) -> String;
    fn bucket(&self) -> String;
    fn key_id(&self) -> String;
    fn key_secret(&self) -> String;
}

pub trait API {
    fn key_urlencode<S: AsRef<str>>(&self, key: S) -> String {
        key.as_ref()
            .split("/")
            .map(|x| urlencoding::encode(x))
            .collect::<Vec<_>>()
            .join("/")
    }

    fn format_key<S: AsRef<str>>(&self, key: S) -> String {
        let key = key.as_ref();
        if key.starts_with("/") {
            key.to_string()
        } else {
            format!("/{}", key)
        }
    }

    fn format_oss_resource_str<S: AsRef<str>>(&self, bucket: S, key: S) -> String;
}

impl OSSInfo for OSS {
    fn endpoint(&self) -> String {
        self.endpoint.clone()
    }
    fn bucket(&self) -> String {
        self.bucket.clone()
    }

    fn key_id(&self) -> String {
        self.key_id.clone()
    }

    fn key_secret(&self) -> String {
        self.key_secret.clone()
    }
}

impl API for OSS {
    fn format_oss_resource_str<S: AsRef<str>>(&self, bucket: S, key: S) -> String {
        let bucket = bucket.as_ref();
        if bucket == "" {
            format!("/{}", bucket)
        } else {
            format!("/{}{}", bucket, key.as_ref())
        }
    }
}

impl<'a> OSS {
    pub fn from_env() -> Self {
        let key_id = std::env::var("OSS_KEY_ID").expect("OSS_KEY_ID not found");
        let key_secret = std::env::var("OSS_KEY_SECRET").expect("OSS_KEY_SECRET not found");
        let endpoint = std::env::var("OSS_ENDPOINT").expect("OSS_ENDPOINT not found");
        let bucket = std::env::var("OSS_BUCKET").expect("OSS_BUCKET not found");
        OSS::new(key_id, key_secret, endpoint, bucket)
    }

    #[cfg(feature = "debug-print")]
    pub fn open_debug(&self) {
        std::env::set_var("RUST_LOG", "oss=debug");
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_line_number(true)
            .init();
    }
    #[cfg(not(feature = "debug-print"))]
    pub fn open_debug(&self) {}
    pub fn new<S: Into<String>>(key_id: S, key_secret: S, endpoint: S, bucket: S) -> Self {
        OSS {
            key_id: key_id.into(),
            key_secret: key_secret.into(),
            endpoint: endpoint.into(),
            bucket: bucket.into(),
        }
    }

    pub fn format_url<S: AsRef<str>>(&self, bucket: S, key: S, build: &RequestBuilder) -> String {
        let key = {
            if build.parameters.len() > 0 {
                let mut params = build.parameters.iter().collect::<Vec<_>>();
                params.sort_by(|a, b| a.0.cmp(&b.0));
                format!(
                    "{}?{}",
                    key.as_ref(),
                    params
                        .into_iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect::<Vec<_>>()
                        .join("&")
                )
            } else {
                key.as_ref().to_string()
            }
        };
        let key = if key.starts_with("/") {
            key
        } else {
            format!("/{}", key)
        };
        if let Some(cdn) = &build.cdn {
            format!("{}{}", cdn, key,)
        } else {
            if self.endpoint().starts_with("https") {
                format!(
                    "https://{}.{}{}",
                    bucket.as_ref(),
                    self.endpoint().replacen("https://", "", 1),
                    key,
                )
            } else {
                format!(
                    "http://{}.{}{}",
                    bucket.as_ref(),
                    self.endpoint().replacen("http://", "", 1),
                    key,
                )
            }
        }
    }

    pub fn build_request<S: AsRef<str>>(
        &self,
        key: S,
        build: RequestBuilder,
    ) -> Result<(String, HeaderMap), InvalidHeaderValue> {
        let mut build = build.clone();
        let url = self.format_url(self.bucket(), key.as_ref().to_string(), &build);
        let mut header = HeaderMap::new();
        let date = self.date();
        header.insert(DATE, date.parse()?);
        build.headers.insert(DATE.to_string(), date);
        let key = key.as_ref();
        let authorization = self.oss_sign(key, &build);
        if let Some(content_type) = build.content_type {
            header.insert(CONTENT_TYPE, content_type.parse()?);
        }
        header.insert(AUTHORIZATION, authorization.parse()?);
        Ok((url, header))
    }
    pub fn date(&self) -> String {
        let now: DateTime<Utc> = Utc::now();
        now.format("%a, %d %b %Y %T GMT").to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::error::OssError;
    use std::io::Read;

    fn open_file(file_name: &str) -> Result<String, OssError> {
        let mut file = std::fs::File::open(file_name)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    }

    #[test]
    fn test_read_file() {
        open_file("a").unwrap();
    }
}
