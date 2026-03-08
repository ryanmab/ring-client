use base64::{Engine, engine::general_purpose::URL_SAFE};
use sha2::{Digest, Sha256};

/// Generates a hardware ID based on the provided system ID.
///
/// The system ID can represent any unique identifier for the system, so long as it is predictable
/// and consistent per device.
///
/// The system ID is used as the hash for the hardware ID, so any changes to the system ID will
/// result in a different hardware ID.
pub fn generate_hardware_id(system_id: &str) -> String {
    let hash = Sha256::digest(system_id.as_bytes());

    URL_SAFE.encode(hash)[..32].to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_generating_hardware_id_is_predictable() {
        let system_id = "mock-system-id";
        let hardware_id = super::generate_hardware_id(system_id);

        assert_eq!(hardware_id, "-LsI4fEAPwq_LcrcqompBfMDMQQltdlQ");
        assert_eq!(hardware_id.len(), 32);
    }
}
