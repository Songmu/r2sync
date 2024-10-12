use std::error::Error;
use url::Url;

#[derive(Debug)]
pub struct R2Location {
    pub bucket: String,
    pub key_prefix: String,
}

pub fn parse_r2_url(r2_url: &str) -> Result<R2Location, Box<dyn Error>> {
    let url = Url::parse(r2_url)?;

    if url.scheme() != "r2" {
        return Err("URL must start with r2://".into());
    }

    let bucket = url.host_str().ok_or("Invalid bucket name")?.to_string();
    let key_prefix = url.path().trim_start_matches('/').to_string();

    Ok(R2Location { bucket, key_prefix })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_r2_url() {
        struct TestCase<'a> {
            name: &'a str,
            input: &'a str,
            expected_bucket: Option<&'a str>,
            expected_key_prefix: Option<&'a str>,
            expected_error: Option<&'a str>,
        }

        let cases = vec![
            TestCase {
                name: "Valid URL with key prefix",
                input: "r2://bucket.example.com/dir/subdir",
                expected_bucket: Some("bucket.example.com"),
                expected_key_prefix: Some("dir/subdir"),
                expected_error: None,
            },
            TestCase {
                name: "Valid URL with no key prefix",
                input: "r2://bucket.example.com/",
                expected_bucket: Some("bucket.example.com"),
                expected_key_prefix: Some(""),
                expected_error: None,
            },
            TestCase {
                name: "Invalid scheme",
                input: "https://bucket.example.com/dir/subdir",
                expected_bucket: None,
                expected_key_prefix: None,
                expected_error: Some("URL must start with r2://"),
            },
            TestCase {
                name: "Missing bucket name",
                input: "r2:///dir/subdir",
                expected_bucket: None,
                expected_key_prefix: None,
                expected_error: Some("Invalid bucket name"),
            },
        ];

        for case in cases {
            let result = parse_r2_url(case.input);

            match case.expected_error {
                Some(expected_error) => {
                    assert!(
                        result.is_err(),
                        "Test '{}' failed: expected an error",
                        case.name
                    );
                    assert_eq!(
                        result.unwrap_err().to_string(),
                        expected_error,
                        "Test '{}' failed: error message mismatch",
                        case.name
                    );
                }
                None => {
                    let result = result.unwrap();
                    assert_eq!(
                        result.bucket,
                        case.expected_bucket.unwrap(),
                        "Test '{}' failed: bucket mismatch",
                        case.name
                    );
                    assert_eq!(
                        result.key_prefix,
                        case.expected_key_prefix.unwrap(),
                        "Test '{}' failed: key prefix mismatch",
                        case.name
                    );
                }
            }
        }
    }
}
