use std::fmt::{Display, Formatter};

/// The Operating System to identify as when beginning a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum OperatingSystem {
    Android,
    Ios,
}

impl OperatingSystem {
    #[must_use]
    pub(crate) const fn get_client_id(self) -> &'static str {
        match self {
            Self::Android => "ring_official_android",
            Self::Ios => "ring_official_ios",
        }
    }

    #[must_use]
    pub(crate) const fn get_user_agent(self) -> &'static str {
        match self {
            Self::Android => "android:com.ringapp",
            Self::Ios => "ios:com.ringapp",
        }
    }
}

impl Display for OperatingSystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Android => f.write_str("android"),
            Self::Ios => f.write_str("ios"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_android() {
        assert_eq!(
            OperatingSystem::Android.get_client_id(),
            "ring_official_android"
        );
        assert_eq!(
            OperatingSystem::Android.get_user_agent(),
            "android:com.ringapp"
        );
    }

    #[test]
    fn test_ios() {
        assert_eq!(OperatingSystem::Ios.get_client_id(), "ring_official_ios");
        assert_eq!(OperatingSystem::Ios.get_user_agent(), "ios:com.ringapp");
    }
}
