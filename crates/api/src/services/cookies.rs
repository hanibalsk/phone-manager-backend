//! Cookie helper module for httpOnly authentication.
//!
//! Provides utilities for setting, reading, and clearing authentication cookies
//! for the admin-portal's browser-based authentication.

use axum::http::{header::SET_COOKIE, HeaderMap, HeaderValue};

use crate::config::CookieConfig;

/// Cookie helper for managing httpOnly authentication cookies.
#[derive(Debug, Clone)]
pub struct CookieHelper {
    config: CookieConfig,
    /// Access token expiry in seconds (from JWT config)
    access_token_expiry_secs: i64,
    /// Refresh token expiry in seconds (from JWT config)
    refresh_token_expiry_secs: i64,
}

impl CookieHelper {
    /// Create a new cookie helper with configuration.
    pub fn new(
        config: CookieConfig,
        access_token_expiry_secs: i64,
        refresh_token_expiry_secs: i64,
    ) -> Self {
        Self {
            config,
            access_token_expiry_secs,
            refresh_token_expiry_secs,
        }
    }

    /// Check if cookie authentication is enabled.
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Build a Set-Cookie header value for the access token.
    pub fn build_access_token_cookie(&self, token: &str) -> String {
        self.build_cookie(
            &self.config.access_token_name,
            token,
            &self.config.access_token_path,
            self.access_token_expiry_secs,
        )
    }

    /// Build a Set-Cookie header value for the refresh token.
    pub fn build_refresh_token_cookie(&self, token: &str) -> String {
        self.build_cookie(
            &self.config.refresh_token_name,
            token,
            &self.config.refresh_token_path,
            self.refresh_token_expiry_secs,
        )
    }

    /// Build a Set-Cookie header to clear the access token cookie.
    pub fn build_clear_access_token_cookie(&self) -> String {
        self.build_clear_cookie(
            &self.config.access_token_name,
            &self.config.access_token_path,
        )
    }

    /// Build a Set-Cookie header to clear the refresh token cookie.
    pub fn build_clear_refresh_token_cookie(&self) -> String {
        self.build_clear_cookie(
            &self.config.refresh_token_name,
            &self.config.refresh_token_path,
        )
    }

    /// Add token cookies to a HeaderMap.
    pub fn add_token_cookies(
        &self,
        headers: &mut HeaderMap,
        access_token: &str,
        refresh_token: &str,
    ) {
        if !self.config.enabled {
            return;
        }

        let access_cookie = self.build_access_token_cookie(access_token);
        let refresh_cookie = self.build_refresh_token_cookie(refresh_token);

        if let Ok(value) = HeaderValue::from_str(&access_cookie) {
            headers.append(SET_COOKIE, value);
        }
        if let Ok(value) = HeaderValue::from_str(&refresh_cookie) {
            headers.append(SET_COOKIE, value);
        }
    }

    /// Add clear cookies to a HeaderMap (for logout).
    pub fn add_clear_cookies(&self, headers: &mut HeaderMap) {
        if !self.config.enabled {
            return;
        }

        let clear_access = self.build_clear_access_token_cookie();
        let clear_refresh = self.build_clear_refresh_token_cookie();

        if let Ok(value) = HeaderValue::from_str(&clear_access) {
            headers.append(SET_COOKIE, value);
        }
        if let Ok(value) = HeaderValue::from_str(&clear_refresh) {
            headers.append(SET_COOKIE, value);
        }
    }

    /// Extract a cookie value from request headers by name.
    pub fn extract_cookie<'a>(&self, headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
        headers
            .get(axum::http::header::COOKIE)
            .and_then(|h| h.to_str().ok())
            .and_then(|cookie_header| {
                cookie_header
                    .split(';')
                    .map(|s| s.trim())
                    .find_map(|cookie| {
                        let (cookie_name, cookie_value) = cookie.split_once('=')?;
                        if cookie_name == name {
                            Some(cookie_value)
                        } else {
                            None
                        }
                    })
            })
    }

    /// Extract the access token from request headers.
    /// Returns the token from the cookie if found.
    pub fn extract_access_token<'a>(&self, headers: &'a HeaderMap) -> Option<&'a str> {
        self.extract_cookie(headers, &self.config.access_token_name)
    }

    /// Extract the refresh token from request headers.
    /// Returns the token from the cookie if found.
    pub fn extract_refresh_token<'a>(&self, headers: &'a HeaderMap) -> Option<&'a str> {
        self.extract_cookie(headers, &self.config.refresh_token_name)
    }

    /// Get the access token name.
    pub fn access_token_name(&self) -> &str {
        &self.config.access_token_name
    }

    /// Get the refresh token name.
    pub fn refresh_token_name(&self) -> &str {
        &self.config.refresh_token_name
    }

    /// Build a cookie string with all security attributes.
    fn build_cookie(&self, name: &str, value: &str, path: &str, max_age: i64) -> String {
        let mut cookie = format!("{}={}; Path={}; Max-Age={}", name, value, path, max_age);

        // Add HttpOnly flag
        cookie.push_str("; HttpOnly");

        // Add Secure flag
        if self.config.secure {
            cookie.push_str("; Secure");
        }

        // Add SameSite attribute
        cookie.push_str(&format!("; SameSite={}", self.config.same_site));

        // Add Domain if configured
        if !self.config.domain.is_empty() {
            cookie.push_str(&format!("; Domain={}", self.config.domain));
        }

        cookie
    }

    /// Build a cookie string that clears an existing cookie.
    fn build_clear_cookie(&self, name: &str, path: &str) -> String {
        let mut cookie = format!(
            "{}=; Path={}; Max-Age=0; Expires=Thu, 01 Jan 1970 00:00:00 GMT",
            name, path
        );

        // Add HttpOnly flag
        cookie.push_str("; HttpOnly");

        // Add Secure flag
        if self.config.secure {
            cookie.push_str("; Secure");
        }

        // Add SameSite attribute
        cookie.push_str(&format!("; SameSite={}", self.config.same_site));

        // Add Domain if configured
        if !self.config.domain.is_empty() {
            cookie.push_str(&format!("; Domain={}", self.config.domain));
        }

        cookie
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> CookieConfig {
        CookieConfig {
            enabled: true,
            secure: true,
            same_site: "Strict".to_string(),
            domain: String::new(),
            access_token_path: "/".to_string(),
            refresh_token_path: "/api/v1/auth".to_string(),
            access_token_name: "access_token".to_string(),
            refresh_token_name: "refresh_token".to_string(),
        }
    }

    #[test]
    fn test_build_access_token_cookie() {
        let helper = CookieHelper::new(test_config(), 3600, 2592000);
        let cookie = helper.build_access_token_cookie("test_token");

        assert!(cookie.contains("access_token=test_token"));
        assert!(cookie.contains("Path=/"));
        assert!(cookie.contains("Max-Age=3600"));
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("Secure"));
        assert!(cookie.contains("SameSite=Strict"));
    }

    #[test]
    fn test_build_refresh_token_cookie() {
        let helper = CookieHelper::new(test_config(), 3600, 2592000);
        let cookie = helper.build_refresh_token_cookie("refresh_test");

        assert!(cookie.contains("refresh_token=refresh_test"));
        assert!(cookie.contains("Path=/api/v1/auth"));
        assert!(cookie.contains("Max-Age=2592000"));
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("Secure"));
        assert!(cookie.contains("SameSite=Strict"));
    }

    #[test]
    fn test_build_clear_cookie() {
        let helper = CookieHelper::new(test_config(), 3600, 2592000);
        let cookie = helper.build_clear_access_token_cookie();

        assert!(cookie.contains("access_token="));
        assert!(cookie.contains("Max-Age=0"));
        assert!(cookie.contains("Expires=Thu, 01 Jan 1970 00:00:00 GMT"));
        assert!(cookie.contains("HttpOnly"));
    }

    #[test]
    fn test_extract_cookie() {
        let helper = CookieHelper::new(test_config(), 3600, 2592000);
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::COOKIE,
            HeaderValue::from_static("access_token=abc123; other=value; refresh_token=xyz789"),
        );

        assert_eq!(helper.extract_access_token(&headers), Some("abc123"));
        assert_eq!(helper.extract_refresh_token(&headers), Some("xyz789"));
    }

    #[test]
    fn test_extract_cookie_not_found() {
        let helper = CookieHelper::new(test_config(), 3600, 2592000);
        let headers = HeaderMap::new();

        assert_eq!(helper.extract_access_token(&headers), None);
        assert_eq!(helper.extract_refresh_token(&headers), None);
    }

    #[test]
    fn test_cookie_with_domain() {
        let mut config = test_config();
        config.domain = "example.com".to_string();

        let helper = CookieHelper::new(config, 3600, 2592000);
        let cookie = helper.build_access_token_cookie("test");

        assert!(cookie.contains("Domain=example.com"));
    }

    #[test]
    fn test_cookie_without_secure() {
        let mut config = test_config();
        config.secure = false;

        let helper = CookieHelper::new(config, 3600, 2592000);
        let cookie = helper.build_access_token_cookie("test");

        assert!(!cookie.contains("Secure"));
    }

    #[test]
    fn test_disabled_helper() {
        let mut config = test_config();
        config.enabled = false;

        let helper = CookieHelper::new(config, 3600, 2592000);
        assert!(!helper.is_enabled());

        let mut headers = HeaderMap::new();
        helper.add_token_cookies(&mut headers, "access", "refresh");
        assert!(headers.get(SET_COOKIE).is_none());
    }
}
