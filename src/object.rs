use anyhow::{anyhow, Result};
use tracing::debug;
use crate::oss::{API, OSS};
use crate::request::{RequestBuilder};

pub trait ObjectAPI {
    /// 获取对象
    ///
    /// # 使用例子
    ///
    /// ```rust
    /// use aliyun_oss_rust_sdk::object::ObjectAPI;
    /// use aliyun_oss_rust_sdk::oss::OSS;
    /// use aliyun_oss_rust_sdk::request::RequestBuilder;
    /// let oss = OSS::from_env();
    /// let build = RequestBuilder::new();
    /// let bytes = oss.get_object("/hello.txt", build).unwrap();
    /// println!("file content: {}", String::from_utf8_lossy(bytes.as_slice()));
    /// ```
    fn get_object<S: AsRef<str>>(
        &self,
        key: S,
        build: &RequestBuilder,
    ) -> Result<Vec<u8>>;
}

impl ObjectAPI for OSS {
    fn get_object<S: AsRef<str>>(&self, key: S, build: &RequestBuilder) -> Result<Vec<u8>> {
        let key = self.format_key(key);
        let (url, headers) = self.build_request(key.as_str(), build)?;
        debug!("get object url: {} headers: {:?}", url,headers);
        let client = reqwest::blocking::Client::new();
        let response = client.get(url)
            .headers(headers).send()?;
        return if response.status().is_success() {
            let result = response.bytes()?;
            Ok(result.to_vec())
        } else {
            let status = response.status();
            let result = response.text()?;
            debug!("get object status: {} error: {}", status,result);
            Err(anyhow!(format!("get object status: {} error: {}", status,result)))
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::object::ObjectAPI;
    use crate::oss::OSS;
    use crate::request::RequestBuilder;

    #[inline]
    fn init_log() {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_line_number(true)
            .init();
    }

    #[test]
    fn test_get_object() {
        init_log();
        let oss = OSS::from_env();
        let build = RequestBuilder::new()
            .with_cdn("http://cdn.ipadump.com");
        let bytes = oss.get_object("/hello.txt", &build).unwrap();
        println!("file content: {}", String::from_utf8_lossy(bytes.as_slice()));
    }
}

