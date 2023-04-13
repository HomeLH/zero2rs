use validator::validate_email;
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
    pub fn parse_use_external_library(s: String) -> Result<SubscriberEmail, String> {
        if !validate_email(&s) {
            return Err(String::from("Invalid email"));
        } else {
            Ok(SubscriberEmail(s))
        }

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
    use fake::{faker::internet::en::SafeEmail, Fake};

    #[derive(Debug, Clone)]
    struct ValidEmailFixtrue(String);

    impl quickcheck::Arbitrary for ValidEmailFixtrue {
        fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
            let email = SafeEmail().fake_with_rng(g);
            Self(email)
        }

    }
    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully(valid_email: ValidEmailFixtrue) -> bool {
        // dbg!
        // dbg!(&valid_email.0);
        SubscriberEmail::parse(valid_email.0).is_ok()
    }

    #[test]
    fn valid_email() {
        let email = SafeEmail().fake();
        assert_ok!(SubscriberEmail::parse(email));
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