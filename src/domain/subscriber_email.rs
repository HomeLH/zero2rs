#[derive(Debug)]
pub struct SubscriberEmail(String);
impl SubscriberEmail {
    pub fn parse(s: String) -> Result<SubscriberEmail, String> {
        if s.is_empty() {
            return Err("Email should not be empty".to_string());
        }
        let parts: Vec<&str> = s.split('@').collect();
        if parts.len() != 2 {
            return Err("Invalid email".to_string());
        }
        let user = parts[0];
        let domain = parts[1];
        if user.is_empty() {
            return Err("Invalid email: no user".to_string());
        }
        if domain.is_empty() {
            return Err("Invalid email: no domain".to_string());
        }
        Ok(SubscriberEmail(s))
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
    
}
#[cfg(test)]
mod tests {
    use super::SubscriberEmail;
    use claim::{assert_ok, assert_err};

    #[test]
    fn valid_email() {
        let email = SubscriberEmail::parse(String::from("example@example.com"));
        assert_ok!(email);
    }

    #[test]
    fn empty_email() {
        let email = SubscriberEmail::parse(String::from(""));
        assert_err!(email);
    }

    #[test]
    fn invalid_email_no_at() {
        let email = SubscriberEmail::parse(String::from("example.com"));
        assert_err!(email);
    }

    #[test]
    fn invalid_email_no_user() {
        let email = SubscriberEmail::parse(String::from("@example.com"));
        assert_err!(email);
    }

    #[test]
    fn invalid_email_no_domain() {
        let email = SubscriberEmail::parse(String::from("example@"));
        assert_err!(email);
    }
}