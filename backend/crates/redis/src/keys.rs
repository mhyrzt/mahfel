/// Namespaced key and channel families.
///
/// Every family lives under the configured prefix so coordination state is
/// identifiable and cannot collide with other applications sharing the
/// instance.
#[derive(Clone, Debug)]
pub struct Keys {
    prefix: String,
}

impl Keys {
    #[must_use]
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
        }
    }

    /// Key for one member of a family, for example `mahfel:presence:<id>`.
    #[must_use]
    pub fn key(&self, family: &str, id: &str) -> String {
        format!("{}:{}:{}", self.prefix, family, id)
    }

    /// Fan-out channel for a family, for example `mahfel:events`.
    #[must_use]
    pub fn channel(&self, family: &str) -> String {
        format!("{}:{}", self.prefix, family)
    }

    #[must_use]
    pub fn prefix(&self) -> &str {
        &self.prefix
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_and_channels_are_namespaced() {
        let keys = Keys::new("mahfel");
        assert_eq!(keys.key("presence", "abc"), "mahfel:presence:abc");
        assert_eq!(keys.channel("events"), "mahfel:events");
        assert_eq!(keys.prefix(), "mahfel");
    }
}
