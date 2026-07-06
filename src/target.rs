use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct SshTarget {
    pub user: String,
    pub host: String,
    pub port: u16,
}

#[derive(Debug, thiserror::Error)]
pub enum TargetParseError {
    #[error("missing '@' separating user and host")]
    MissingAt,
    #[error("missing ':' separating host and port")]
    MissingColon,
    #[error("invalid port: {0}")]
    InvalidPort(#[from] std::num::ParseIntError),
}

impl FromStr for SshTarget {
    type Err = TargetParseError;

    /// Parses `user@host:port`. IPv6 hosts (unbracketed) are not supported here —
    /// use the `/ssh?user=&host=&port=` query form instead.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (user, rest) = s.split_once('@').ok_or(TargetParseError::MissingAt)?;
        let (host, port_str) = rest
            .rsplit_once(':')
            .ok_or(TargetParseError::MissingColon)?;
        let port: u16 = port_str.parse()?;
        Ok(SshTarget {
            user: user.to_string(),
            host: host.to_string(),
            port,
        })
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct SshQuery {
    pub user: String,
    pub host: String,
    pub port: u16,
}

impl From<SshQuery> for SshTarget {
    fn from(q: SshQuery) -> Self {
        SshTarget {
            user: q.user,
            host: q.host,
            port: q.port,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_target() {
        let t: SshTarget = "root@example.com:2222".parse().unwrap();
        assert_eq!(t.user, "root");
        assert_eq!(t.host, "example.com");
        assert_eq!(t.port, 2222);
    }

    #[test]
    fn rejects_missing_at() {
        assert!(matches!(
            "example.com:22".parse::<SshTarget>(),
            Err(TargetParseError::MissingAt)
        ));
    }

    #[test]
    fn rejects_missing_colon() {
        assert!(matches!(
            "root@example.com".parse::<SshTarget>(),
            Err(TargetParseError::MissingColon)
        ));
    }

    #[test]
    fn rejects_bad_port() {
        assert!(matches!(
            "root@example.com:notaport".parse::<SshTarget>(),
            Err(TargetParseError::InvalidPort(_))
        ));
    }
}
