use rand::{
    distr::{Alphanumeric, SampleString},
    rngs::StdRng,
};
use rand_seeder::Seeder;

/// Generates a hardware ID based on the provided system ID.
///
/// The system ID can represent any unique identifier for the system, so long as it is predictable
/// and consistent per device.
///
/// The system ID is used as the seed for the hardware ID, so any changes to the system ID will
/// result in a different hardware ID.
pub fn generate_hardware_id(system_id: &str) -> String {
    let mut rand: StdRng = Seeder::from(system_id).into_rng();

    Alphanumeric.sample_string(&mut rand, 32)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_generating_hardware_id_is_predictable() {
        let system_id = "mock-system-id";
        let hardware_id = super::generate_hardware_id(system_id);

        assert_eq!(hardware_id, "553MdASrIII8WL54vPBBr78lhXUH6WZu");
        assert_eq!(hardware_id.len(), 32);
    }
}
