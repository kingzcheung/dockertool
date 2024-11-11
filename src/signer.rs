use std::collections::HashMap;

use axum::http::{HeaderName, HeaderValue};
use chrono::{DateTime, TimeZone, Utc};
use clap::builder::Str;
use hmac::{Hmac, Mac};
use hyper::HeaderMap;
use sha2::{Digest, Sha256};
use url::form_urlencoded;

const BASIC_DATE_FORMAT: &str = "%Y%m%dT%H%M%SZ";
const ALGORITHM: &str = "SDK-HMAC-SHA256";
const HEADER_X_DATE: &str = "X-Sdk-Date";
const HEADER_HOST: &str = "host";
const HEADER_AUTHORIZATION: &str = "Authorization";
const HEADER_CONTENT_SHA256: &str = "x-sdk-content-sha256";

fn hmac_sha256(key: &str, message: &str) -> Vec<u8> {
    // 创建 HmacSha256 实例
    let mut mac =
        Hmac::<Sha256>::new_from_slice(key.as_bytes()).expect("HMAC can take key of any size");

    // 更新消息
    mac.update(message.as_bytes());

    // 获取结果并返回
    mac.finalize().into_bytes().to_vec()
}

fn hex_encode_sha256_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

fn string_to_sign(canonical_request: &str, t: DateTime<Utc>) -> String {
    let bytes = hex_encode_sha256_hash(canonical_request.as_bytes());
    format!("{}\n{}\n{}", ALGORITHM, t.format(BASIC_DATE_FORMAT), bytes)
}

/// 对字符串进行 URL 编码
fn urlencode(s: &str) -> String {
    form_urlencoded::byte_serialize(s.as_bytes()).collect()
}
/// 在请求头中查找指定的头
fn find_header(headers: &HashMap<String, String>, header: &str) -> Option<String> {
    headers.iter().find_map(|(k, v)| {
        if k.to_lowercase() == header.to_lowercase() {
            Some(v.to_owned())
        } else {
            None
        }
    })
}

#[derive(Debug)]
struct HttpRequest {
    method: String,
    scheme: String,
    host: String,
    uri: String,
    url: String,
    query: HashMap<String, Vec<String>>,
    headers: HashMap<String, String>,
    body: Vec<u8>,
    http_client: reqwest::Client,
}

impl HttpRequest {
    fn new(method: &str, url: &str, headers: Option<HashMap<String, String>>, body: &str) -> Self {
        let mut query = HashMap::new();
        let mut scheme = "http".to_string();
        let mut host = "".to_string();
        let mut uri = "/".to_string();

        // 解析 URL
        let url_parts: Vec<&str> = url.splitn(2, "://").collect();
        if url_parts.len() > 1 {
            scheme = url_parts[0].to_string();
            let path_and_query = url_parts[1];
            let path_query_parts: Vec<&str> = path_and_query.splitn(2, '?').collect();
            let path = path_query_parts[0];
            let path_parts: Vec<&str> = path.splitn(2, '/').collect();
            host = path_parts[0].to_string();
            if path_parts.len() > 1 {
                uri = format!("/{}", path_parts[1]);
            } else {
                uri = "/".to_string();
            }

            if path_query_parts.len() > 1 {
                let query_string = path_query_parts[1];
                for (key, value) in form_urlencoded::parse(query_string.as_bytes()) {
                    query
                        .entry(key.into_owned())
                        .or_insert_with(Vec::new)
                        .push(value.into_owned());
                }
            }
        }

        // 处理 headers
        let headers = headers.unwrap_or_default();

        // 处理 body
        let body = body.as_bytes().to_vec();

        let client = reqwest::Client::new();

        HttpRequest {
            method: method.to_string(),
            scheme,
            host,
            uri,
            url: url.to_string(),
            query,
            headers,
            body,
            http_client: client,
        }
    }

    pub async fn show_repository(&self) {
        dbg!(&self.headers);

        let headers: HeaderMap = self
            .headers
            .iter()
            .map(|(k, v)| {
                (
                    HeaderName::from_bytes(k.as_bytes()).unwrap(),
                    HeaderValue::from_bytes(v.as_bytes()).unwrap(),
                )
            })
            .collect();
        let text = self
            .http_client
            .get(&self.url)
            .headers(headers)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        println!("{}", text);
    }
}

pub struct Signer {
    access_key_id: String,
    access_key_secret: String,
}

impl Signer {
    pub fn new(access_key_id: String, access_key_secret: String) -> Self {
        Self {
            access_key_id,
            access_key_secret,
        }
    }

    pub fn sign(&self, r: &mut HttpRequest) {
        let heder_time = find_header(&r.headers, HEADER_X_DATE);

        let t = match heder_time {
            Some(t) => t.parse::<DateTime<Utc>>().unwrap(),
            None => {
                let t = Utc::now();
                r.headers.insert(
                    HEADER_X_DATE.to_string(),
                    t.format(BASIC_DATE_FORMAT).to_string(),
                );
                t
            }
        };

        let mut have_host = false;
        for key in r.headers.keys() {
            if key.to_lowercase() == "host" {
                have_host = true;
                break;
            }
        }
        if !have_host {
            r.headers.insert(HEADER_HOST.to_string(), r.host.clone());
        }

        let signed_headers = signed_headers(&r.headers);

        let canonical_request = canonical_request(r, &signed_headers);

        let string_to_sign = string_to_sign(&canonical_request, t);
        let signature = sign_string_to_sign(&string_to_sign, &self.access_key_secret);
        let auth_value = auth_header_value(&signature, &self.access_key_id, &signed_headers);
        r.headers
            .insert(HEADER_AUTHORIZATION.to_string(), auth_value);
        if !r.body.is_empty() {
            r.headers
                .insert("content-length".to_owned(), r.body.len().to_string());
        }

        let query_string = canonical_query_string(&r.query);
        if !query_string.is_empty() {
            r.uri = r.uri.clone() + "?" + &query_string;
        }
    }
}

/// 生成认证头值
fn auth_header_value(signature: &str, app_key: &str, signed_headers: &[String]) -> String {
    format!(
        "{} Access={}, SignedHeaders={}, Signature={}",
        ALGORITHM,
        app_key,
        signed_headers.join(";"),
        signature
    )
}

/// 创建 HWS 签名
fn sign_string_to_sign(string_to_sign: &str, signing_key: &str) -> String {
    // 创建 HmacSha256 实例
    let mut mac = Hmac::<Sha256>::new_from_slice(signing_key.as_bytes())
        .expect("HMAC can take key of any size");

    // 更新消息
    mac.update(string_to_sign.as_bytes());

    // 获取结果并返回
    let result = mac.finalize().into_bytes();
    hex::encode(result)
}

fn canonical_request(req: &mut HttpRequest, signed_headers: &[String]) -> String {
    let canonical_header = canonical_header(req, signed_headers);
    let hexencode = find_header(&req.headers, HEADER_CONTENT_SHA256)
        .unwrap_or_else(|| hex_encode_sha256_hash(&req.body));

    format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        req.method.to_uppercase(),
        canonical_uri(&req.uri),
        canonical_query_string(&req.query),
        canonical_header,
        signed_headers.join(";"),
        hexencode
    )
}

fn canonical_header(r: &mut HttpRequest, signed_headers: &[String]) -> String {
    let mut a = Vec::new();
    let mut __headers = HashMap::new();

    for (key, value) in &mut r.headers {
        let key_encoded = key.to_lowercase();
        let value_encoded = value.trim().to_string();
        __headers.insert(key_encoded.clone(), value_encoded.clone());
        // r.headers.insert(key.clone(), value_encoded);
    }
    for key in signed_headers {
        if let Some(value) = __headers.get(key) {
            a.push(format!("{}:{}", key, value));
        }
    }

    for (k, v) in __headers {
        r.headers.insert(k, v);
    }

    a.join("\n") + "\n"
}

fn canonical_query_string(query: &HashMap<String, Vec<String>>) -> String {
    let mut keys: Vec<&String> = query.keys().collect();
    keys.sort();

    let mut a = Vec::new();
    for key in keys {
        let k: String = form_urlencoded::byte_serialize(key.as_bytes()).collect::<String>();
        let mut value = query.get(key).map(|x| x.to_owned()).unwrap();
        value.sort();
        for v in value {
            let kv = format!("{}={}", k, urlencode(&v));
            a.push(kv);
        }
    }

    a.join("&")
}

fn canonical_uri(r: &str) -> String {
    let patterns: Vec<String> = urlencoding::decode(r)
        .unwrap()
        .split('/')
        .map(|v| v.to_string())
        .collect();

    let mut uri: Vec<String> = Vec::new();
    for v in patterns {
        uri.push(urlencode(&v));
    }

    let mut urlpath = uri.join("/");
    if !urlpath.ends_with('/') {
        urlpath.push('/');
    }

    urlpath
}

fn signed_headers(headers: &HashMap<String, String>) -> Vec<String> {
    let mut keys: Vec<_> = headers.keys().cloned().collect();
    keys.sort_unstable_by_key(|a| a.to_lowercase());
    keys
}

#[cfg(test)]
mod test {
    use std::env;

    use super::*;

    #[tokio::test]
    async fn test_signer() {
        dotenvy::dotenv().unwrap();

        let access_key_id = env::var("OBS_AK").unwrap();
        let access_key_secret = env::var("OBS_SK").unwrap();
        let sign = Signer::new(access_key_id, access_key_secret);
        let url = format!(
            "https://swr-api.cn-south-1.myhuaweicloud.com/v2/manage/namespaces/{namespace}/repos/{repository}",
            namespace = "czking",
            repository = "alpine"
        );
        dbg!(&url);
        let headers = HashMap::from([("Content-Type".to_string(), "application/json".to_string())]);
        let mut r = HttpRequest::new("GET", &url, Some(headers), "");
        sign.sign(&mut r);

        r.show_repository().await;
    }
}
