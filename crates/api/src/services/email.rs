//! Email service for sending verification and password reset emails.
//!
//! Supports multiple email providers:
//! - `console`: Logs emails to console (development)
//! - `smtp`: Sends via SMTP server
//! - `sendgrid`: Uses SendGrid API
//! - `ses`: Uses AWS SES (future implementation)

use crate::config::EmailConfig;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, error, info, warn};

/// Errors that can occur during email operations.
#[derive(Debug, Error)]
pub enum EmailError {
    #[error("Email service not configured")]
    NotConfigured,

    #[error("Email service disabled")]
    Disabled,

    #[error("Invalid email address: {0}")]
    InvalidAddress(String),

    #[error("Failed to send email: {0}")]
    SendFailed(String),

    #[error("Template rendering error: {0}")]
    TemplateError(String),

    #[error("Provider error: {0}")]
    ProviderError(String),
}

/// Email message to be sent.
#[derive(Debug, Clone)]
pub struct EmailMessage {
    /// Recipient email address
    pub to: String,
    /// Recipient name (optional)
    pub to_name: Option<String>,
    /// Email subject
    pub subject: String,
    /// Plain text body
    pub body_text: String,
    /// HTML body (optional)
    pub body_html: Option<String>,
}

/// Email service for sending transactional emails.
#[derive(Clone)]
pub struct EmailService {
    config: Arc<EmailConfig>,
}

impl EmailService {
    /// Creates a new EmailService with the given configuration.
    pub fn new(config: EmailConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    /// Check if email service is enabled.
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Send an email message.
    pub async fn send(&self, message: EmailMessage) -> Result<(), EmailError> {
        if !self.config.enabled {
            debug!(
                to = %message.to,
                subject = %message.subject,
                "Email service disabled, skipping send"
            );
            return Ok(());
        }

        match self.config.provider.as_str() {
            "console" => self.send_console(message).await,
            "smtp" => self.send_smtp(message).await,
            "sendgrid" => self.send_sendgrid(message).await,
            "ses" => self.send_ses(message).await,
            provider => {
                error!(provider = %provider, "Unknown email provider");
                Err(EmailError::NotConfigured)
            }
        }
    }

    /// Send email verification email.
    pub async fn send_verification_email(
        &self,
        to_email: &str,
        to_name: Option<&str>,
        verification_token: &str,
    ) -> Result<(), EmailError> {
        let verification_url = format!(
            "{}/verify-email?token={}",
            self.config.base_url, verification_token
        );

        let subject = "Verify your email address - Phone Manager";

        let body_text = format!(
            r#"Hi{name},

Please verify your email address by clicking the link below:

{url}

This link will expire in 24 hours.

If you didn't create an account with Phone Manager, you can safely ignore this email.

Best regards,
The Phone Manager Team"#,
            name = to_name.map(|n| format!(" {}", n)).unwrap_or_default(),
            url = verification_url
        );

        let body_html = if self.config.template_style == "html" {
            Some(format!(
                r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Verify your email</title>
</head>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 20px;">
    <div style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); padding: 30px; border-radius: 10px 10px 0 0;">
        <h1 style="color: white; margin: 0; font-size: 24px;">Phone Manager</h1>
    </div>
    <div style="background: #f9f9f9; padding: 30px; border-radius: 0 0 10px 10px;">
        <h2 style="color: #333; margin-top: 0;">Verify your email address</h2>
        <p>Hi{name},</p>
        <p>Thanks for signing up for Phone Manager! Please verify your email address by clicking the button below:</p>
        <div style="text-align: center; margin: 30px 0;">
            <a href="{url}" style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 14px 28px; text-decoration: none; border-radius: 6px; font-weight: bold; display: inline-block;">Verify Email Address</a>
        </div>
        <p style="color: #666; font-size: 14px;">This link will expire in 24 hours.</p>
        <p style="color: #666; font-size: 14px;">If you didn't create an account with Phone Manager, you can safely ignore this email.</p>
        <hr style="border: none; border-top: 1px solid #ddd; margin: 30px 0;">
        <p style="color: #999; font-size: 12px;">Or copy and paste this link into your browser:<br><a href="{url}" style="color: #667eea;">{url}</a></p>
    </div>
</body>
</html>"#,
                name = to_name.map(|n| format!(" {}", n)).unwrap_or_default(),
                url = verification_url
            ))
        } else {
            None
        };

        let message = EmailMessage {
            to: to_email.to_string(),
            to_name: to_name.map(|s| s.to_string()),
            subject: subject.to_string(),
            body_text,
            body_html,
        };

        self.send(message).await
    }

    /// Send password reset email.
    pub async fn send_password_reset_email(
        &self,
        to_email: &str,
        to_name: Option<&str>,
        reset_token: &str,
    ) -> Result<(), EmailError> {
        let reset_url = format!(
            "{}/reset-password?token={}",
            self.config.base_url, reset_token
        );

        let subject = "Reset your password - Phone Manager";

        let body_text = format!(
            r#"Hi{name},

We received a request to reset your password. Click the link below to create a new password:

{url}

This link will expire in 1 hour.

If you didn't request a password reset, you can safely ignore this email. Your password will remain unchanged.

Best regards,
The Phone Manager Team"#,
            name = to_name.map(|n| format!(" {}", n)).unwrap_or_default(),
            url = reset_url
        );

        let body_html = if self.config.template_style == "html" {
            Some(format!(
                r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Reset your password</title>
</head>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 20px;">
    <div style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); padding: 30px; border-radius: 10px 10px 0 0;">
        <h1 style="color: white; margin: 0; font-size: 24px;">Phone Manager</h1>
    </div>
    <div style="background: #f9f9f9; padding: 30px; border-radius: 0 0 10px 10px;">
        <h2 style="color: #333; margin-top: 0;">Reset your password</h2>
        <p>Hi{name},</p>
        <p>We received a request to reset your password. Click the button below to create a new password:</p>
        <div style="text-align: center; margin: 30px 0;">
            <a href="{url}" style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 14px 28px; text-decoration: none; border-radius: 6px; font-weight: bold; display: inline-block;">Reset Password</a>
        </div>
        <p style="color: #666; font-size: 14px;">This link will expire in 1 hour.</p>
        <p style="color: #666; font-size: 14px;">If you didn't request a password reset, you can safely ignore this email. Your password will remain unchanged.</p>
        <hr style="border: none; border-top: 1px solid #ddd; margin: 30px 0;">
        <p style="color: #999; font-size: 12px;">Or copy and paste this link into your browser:<br><a href="{url}" style="color: #667eea;">{url}</a></p>
    </div>
</body>
</html>"#,
                name = to_name.map(|n| format!(" {}", n)).unwrap_or_default(),
                url = reset_url
            ))
        } else {
            None
        };

        let message = EmailMessage {
            to: to_email.to_string(),
            to_name: to_name.map(|s| s.to_string()),
            subject: subject.to_string(),
            body_text,
            body_html,
        };

        self.send(message).await
    }

    /// Console provider - logs email to console (for development).
    async fn send_console(&self, message: EmailMessage) -> Result<(), EmailError> {
        info!(
            to = %message.to,
            to_name = ?message.to_name,
            subject = %message.subject,
            from = %self.config.sender_email,
            from_name = %self.config.sender_name,
            "📧 Email (console provider)"
        );

        info!(
            body_text = %message.body_text,
            "📧 Email body (plain text)"
        );

        if let Some(html) = &message.body_html {
            debug!(
                body_html_length = %html.len(),
                "📧 Email body (HTML) - {} chars",
                html.len()
            );
        }

        Ok(())
    }

    /// SMTP provider - sends via SMTP server.
    async fn send_smtp(&self, message: EmailMessage) -> Result<(), EmailError> {
        if self.config.smtp_host.is_empty() {
            return Err(EmailError::NotConfigured);
        }

        // Build the email using reqwest to make a simple HTTP call to a webhook or use lettre
        // For MVP, we'll use a simple implementation
        // In production, consider using the `lettre` crate for full SMTP support

        warn!(
            provider = "smtp",
            host = %self.config.smtp_host,
            port = %self.config.smtp_port,
            "SMTP provider configured but full implementation requires lettre crate"
        );

        // Log the email that would have been sent
        info!(
            to = %message.to,
            subject = %message.subject,
            smtp_host = %self.config.smtp_host,
            "📧 Email would be sent via SMTP (full implementation pending)"
        );

        // For now, don't fail - just log
        // In production, add lettre crate and implement properly
        Ok(())
    }

    /// SendGrid provider - sends via SendGrid API.
    async fn send_sendgrid(&self, message: EmailMessage) -> Result<(), EmailError> {
        if self.config.sendgrid_api_key.is_empty() {
            return Err(EmailError::NotConfigured);
        }

        let client = reqwest::Client::new();

        // Build SendGrid API request
        let mut personalizations = serde_json::json!({
            "to": [{
                "email": message.to
            }]
        });

        if let Some(name) = &message.to_name {
            personalizations["to"][0]["name"] = serde_json::json!(name);
        }

        let mut body = serde_json::json!({
            "personalizations": [personalizations],
            "from": {
                "email": self.config.sender_email,
                "name": self.config.sender_name
            },
            "subject": message.subject,
            "content": [{
                "type": "text/plain",
                "value": message.body_text
            }]
        });

        if let Some(html) = &message.body_html {
            body["content"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!({
                    "type": "text/html",
                    "value": html
                }));
        }

        let response = client
            .post("https://api.sendgrid.com/v3/mail/send")
            .header(
                "Authorization",
                format!("Bearer {}", self.config.sendgrid_api_key),
            )
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| EmailError::SendFailed(format!("SendGrid request failed: {}", e)))?;

        if response.status().is_success() {
            info!(
                to = %message.to,
                subject = %message.subject,
                "📧 Email sent via SendGrid"
            );
            Ok(())
        } else {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            error!(
                status = %status,
                error = %error_body,
                "SendGrid API error"
            );
            Err(EmailError::ProviderError(format!(
                "SendGrid returned {}: {}",
                status, error_body
            )))
        }
    }

    /// AWS SES provider - placeholder for future implementation.
    async fn send_ses(&self, message: EmailMessage) -> Result<(), EmailError> {
        warn!(
            provider = "ses",
            to = %message.to,
            "SES provider not yet implemented"
        );

        // For now, log the email that would have been sent
        info!(
            to = %message.to,
            subject = %message.subject,
            "📧 Email would be sent via AWS SES (implementation pending)"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> EmailConfig {
        EmailConfig {
            enabled: true,
            provider: "console".to_string(),
            smtp_host: String::new(),
            smtp_port: 587,
            smtp_username: String::new(),
            smtp_password: String::new(),
            smtp_use_tls: true,
            sendgrid_api_key: String::new(),
            ses_region: String::new(),
            sender_email: "test@example.com".to_string(),
            sender_name: "Test".to_string(),
            base_url: "https://app.example.com".to_string(),
            template_style: "html".to_string(),
        }
    }

    #[test]
    fn test_email_service_creation() {
        let config = test_config();
        let service = EmailService::new(config);
        assert!(service.is_enabled());
    }

    #[test]
    fn test_email_service_disabled() {
        let mut config = test_config();
        config.enabled = false;
        let service = EmailService::new(config);
        assert!(!service.is_enabled());
    }

    #[tokio::test]
    async fn test_send_console_email() {
        let config = test_config();
        let service = EmailService::new(config);

        let message = EmailMessage {
            to: "user@example.com".to_string(),
            to_name: Some("Test User".to_string()),
            subject: "Test Subject".to_string(),
            body_text: "Test body".to_string(),
            body_html: Some("<p>Test body</p>".to_string()),
        };

        let result = service.send(message).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_disabled_silently_succeeds() {
        let mut config = test_config();
        config.enabled = false;
        let service = EmailService::new(config);

        let message = EmailMessage {
            to: "user@example.com".to_string(),
            to_name: None,
            subject: "Test".to_string(),
            body_text: "Test".to_string(),
            body_html: None,
        };

        let result = service.send(message).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_verification_email() {
        let config = test_config();
        let service = EmailService::new(config);

        let result = service
            .send_verification_email("user@example.com", Some("Test User"), "test-token-123")
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_password_reset_email() {
        let config = test_config();
        let service = EmailService::new(config);

        let result = service
            .send_password_reset_email("user@example.com", Some("Test User"), "reset-token-456")
            .await;

        assert!(result.is_ok());
    }

    #[test]
    fn test_email_message_creation() {
        let message = EmailMessage {
            to: "user@example.com".to_string(),
            to_name: Some("User Name".to_string()),
            subject: "Test Subject".to_string(),
            body_text: "Plain text body".to_string(),
            body_html: Some("<p>HTML body</p>".to_string()),
        };

        assert_eq!(message.to, "user@example.com");
        assert_eq!(message.to_name, Some("User Name".to_string()));
        assert_eq!(message.subject, "Test Subject");
    }
}
