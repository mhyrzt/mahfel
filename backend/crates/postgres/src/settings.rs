use crate::DatabaseError;
use std::time::Duration;

/// Connection pool sizing and lifetime policy.
#[derive(Clone, Copy, Debug)]
pub struct PoolSettings {
    pub min_connections: u32,
    pub max_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
}

/// Network and credential details for one PostgreSQL database.
#[derive(Clone)]
pub struct ConnectionSettings {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub name: String,
}

impl PoolSettings {
    pub fn validate(self) -> Result<Self, DatabaseError> {
        if self.max_connections == 0 {
            return Err(DatabaseError::InvalidConfiguration(
                "max_connections must be greater than zero",
            ));
        }
        if self.min_connections > self.max_connections {
            return Err(DatabaseError::InvalidConfiguration(
                "min_connections must not exceed max_connections",
            ));
        }
        if self.acquire_timeout.is_zero() {
            return Err(DatabaseError::InvalidConfiguration(
                "acquire_timeout must be greater than zero",
            ));
        }
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_settings() -> PoolSettings {
        PoolSettings {
            min_connections: 1,
            max_connections: 4,
            acquire_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(60),
            max_lifetime: Duration::from_secs(300),
        }
    }

    #[test]
    fn rejects_an_inverted_connection_range() {
        let settings = PoolSettings {
            min_connections: 5,
            max_connections: 4,
            ..valid_settings()
        };
        assert!(settings.validate().is_err());
    }

    #[test]
    fn rejects_zero_acquire_timeout() {
        let settings = PoolSettings {
            acquire_timeout: Duration::ZERO,
            ..valid_settings()
        };
        assert!(settings.validate().is_err());
    }

    #[test]
    fn accepts_valid_settings() {
        assert!(valid_settings().validate().is_ok());
    }
}
