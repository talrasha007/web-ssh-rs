use std::str::FromStr;

const DEFAULT_SSH_PORT: u16 = 22;

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
    #[error("invalid port: {0}")]
    InvalidPort(#[from] std::num::ParseIntError),
}

impl FromStr for SshTarget {
    type Err = TargetParseError;

    /// Parses `user@host` or `user@host:port` (port defaults to 22 if omitted).
    /// IPv6 hosts (unbracketed) are not supported here — use the
    /// `/ssh?user=&host=&port=` query form instead.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (user, rest) = s.split_once('@').ok_or(TargetParseError::MissingAt)?;
        let (host, port) = match rest.rsplit_once(':') {
            Some((host, port_str)) => (host, port_str.parse()?),
            None => (rest, DEFAULT_SSH_PORT),
        };
        Ok(SshTarget {
            user: user.to_string(),
            host: host.to_string(),
            port,
        })
    }
}

fn default_ssh_port() -> u16 {
    DEFAULT_SSH_PORT
}

#[derive(Debug, serde::Deserialize)]
pub struct SshQuery {
    pub user: String,
    pub host: String,
    #[serde(default = "default_ssh_port")]
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
    fn defaults_to_port_22_when_omitted() {
        let t: SshTarget = "root@example.com".parse().unwrap();
        assert_eq!(t.user, "root");
        assert_eq!(t.host, "example.com");
        assert_eq!(t.port, 22);
    }

    #[test]
    fn rejects_missing_at() {
        assert!(matches!(
            "example.com:22".parse::<SshTarget>(),
            Err(TargetParseError::MissingAt)
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
