use crate::bitflags_serde_impl;
use crate::models::ids::PatId;
use ariadne::ids::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

bitflags::bitflags! {
    #[derive(Copy, Clone, Debug)]
    pub struct Scopes: u64 {
        // read a user's email
        const USER_READ_EMAIL = 1 << 0;
        // read a user's data
        const USER_READ = 1 << 1;
        // write to a user's profile (edit username, email, avatar, follows, etc)
        const USER_WRITE = 1 << 2;
        // delete a user
        const USER_DELETE = 1 << 3;
        // modify a user's authentication data
        const USER_AUTH_WRITE = 1 << 4;

        // read a user's notifications
        const NOTIFICATION_READ = 1 << 5;
        // delete or read a notification
        const NOTIFICATION_WRITE = 1 << 6;

        // read a user's payouts data
        const PAYOUTS_READ = 1 << 7;
        // withdraw money from a user's account
        const PAYOUTS_WRITE = 1<< 8;
        // access user analytics (payout analytics at the moment)
        const ANALYTICS = 1 << 9;

        // create a project
        const PROJECT_CREATE = 1 << 10;
        // read a user's projects (including private)
        const PROJECT_READ = 1 << 11;
        // write to a project's data (metadata, title, team members, etc)
        const PROJECT_WRITE = 1 << 12;
        // delete a project
        const PROJECT_DELETE = 1 << 13;

        // create a version
        const VERSION_CREATE = 1 << 14;
        // read a user's versions (including private)
        const VERSION_READ = 1 << 15;
        // write to a version's data (metadata, files, etc)
        const VERSION_WRITE = 1 << 16;
        // delete a version
        const VERSION_DELETE = 1 << 17;

        // create a report
        const REPORT_CREATE = 1 << 18;
        // read a user's reports
        const REPORT_READ = 1 << 19;
        // edit a report
        const REPORT_WRITE = 1 << 20;
        // delete a report
        const REPORT_DELETE = 1 << 21;

        // read a thread
        const THREAD_READ = 1 << 22;
        // write to a thread (send a message, delete a message)
        const THREAD_WRITE = 1 << 23;

        // create a pat
        const PAT_CREATE = 1 << 24;
        // read a user's pats
        const PAT_READ = 1 << 25;
        // edit a pat
        const PAT_WRITE = 1 << 26;
        // delete a pat
        const PAT_DELETE = 1 << 27;

        // read a user's sessions
        const SESSION_READ = 1 << 28;
        // delete a session
        const SESSION_DELETE = 1 << 29;

        // perform analytics action
        const PERFORM_ANALYTICS = 1 << 30;

        // create a collection
        const COLLECTION_CREATE = 1 << 31;
        // read a user's collections
        const COLLECTION_READ = 1 << 32;
        // write to a collection
        const COLLECTION_WRITE = 1 << 33;
        // delete a collection
        const COLLECTION_DELETE = 1 << 34;

        // create an organization
        const ORGANIZATION_CREATE = 1 << 35;
        // read a user's organizations
        const ORGANIZATION_READ = 1 << 36;
        // write to an organization
        const ORGANIZATION_WRITE = 1 << 37;
        // delete an organization
        const ORGANIZATION_DELETE = 1 << 38;

        // only accessible by modrinth-issued sessions
        const SESSION_ACCESS = 1 << 39;

        const NONE = 0b0;
    }
}

bitflags_serde_impl!(Scopes, u64);

impl Scopes {
    // these scopes cannot be specified in a personal access token
    pub fn restricted() -> Scopes {
        Scopes::PAT_CREATE
            | Scopes::PAT_READ
            | Scopes::PAT_WRITE
            | Scopes::PAT_DELETE
            | Scopes::SESSION_READ
            | Scopes::SESSION_DELETE
            | Scopes::SESSION_ACCESS
            | Scopes::USER_AUTH_WRITE
            | Scopes::USER_DELETE
            | Scopes::PERFORM_ANALYTICS
    }

    pub fn is_restricted(&self) -> bool {
        self.intersects(Self::restricted())
    }

    pub fn parse_from_oauth_scopes(
        scopes: &str,
    ) -> Result<Scopes, bitflags::parser::ParseError> {
        let scopes = scopes.replace(['+', ' '], "|").replace("%20", "|");
        bitflags::parser::from_str(&scopes)
    }

    pub fn to_postgres(&self) -> i64 {
        self.bits() as i64
    }

    pub fn from_postgres(value: i64) -> Self {
        Self::from_bits(value as u64).unwrap_or(Scopes::NONE)
    }
}

#[derive(Serialize, Deserialize)]
pub struct PersonalAccessToken {
    pub id: PatId,
    pub name: String,
    pub access_token: Option<String>,
    pub scopes: Scopes,
    pub user_id: UserId,
    pub created: DateTime<Utc>,
    pub expires: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
}

impl PersonalAccessToken {
    pub fn from(
        data: crate::database::models::pat_item::DBPersonalAccessToken,
        include_token: bool,
    ) -> Self {
        Self {
            id: data.id.into(),
            name: data.name,
            access_token: if include_token {
                Some(data.access_token)
            } else {
                None
            },
            scopes: data.scopes,
            user_id: data.user_id.into(),
            created: data.created,
            expires: data.expires,
            last_used: data.last_used,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use itertools::Itertools;

    #[test]
    fn test_parse_from_oauth_scopes_well_formed() {
        let raw = "USER_READ_EMAIL SESSION_READ ORGANIZATION_CREATE";
        let expected = Scopes::USER_READ_EMAIL
            | Scopes::SESSION_READ
            | Scopes::ORGANIZATION_CREATE;

        let parsed = Scopes::parse_from_oauth_scopes(raw).unwrap();

        assert_same_flags(expected, parsed);
    }

    #[test]
    fn test_parse_from_oauth_scopes_empty() {
        let raw = "";
        let expected = Scopes::empty();

        let parsed = Scopes::parse_from_oauth_scopes(raw).unwrap();

        assert_same_flags(expected, parsed);
    }

    #[test]
    fn test_parse_from_oauth_scopes_invalid_scopes() {
        let raw = "notascope";

        let parsed = Scopes::parse_from_oauth_scopes(raw);

        assert!(parsed.is_err());
    }

    #[test]
    fn test_parse_from_oauth_scopes_invalid_separator() {
        let raw = "USER_READ_EMAIL & SESSION_READ";

        let parsed = Scopes::parse_from_oauth_scopes(raw);

        assert!(parsed.is_err());
    }

    #[test]
    fn test_parse_from_oauth_scopes_url_encoded() {
        let raw =
            urlencoding::encode("PAT_WRITE COLLECTION_DELETE").to_string();
        let expected = Scopes::PAT_WRITE | Scopes::COLLECTION_DELETE;

        let parsed = Scopes::parse_from_oauth_scopes(&raw).unwrap();

        assert_same_flags(expected, parsed);
    }

    fn assert_same_flags(expected: Scopes, actual: Scopes) {
        assert_eq!(
            expected.iter_names().map(|(name, _)| name).collect_vec(),
            actual.iter_names().map(|(name, _)| name).collect_vec()
        );
    }
}
