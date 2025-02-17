//! Sample test vectors for delegation deploys.
//!
//! Method name (entrypoint):
//! `redelegate`
//!
//! Arguments:
//! | name | type |
//! |---------|---------|
//! | `delegator` | `PublicKey` |
//! | `validator` | `PublicKey` |
//! | `new_validator` | `PublicKey` |
//! | `amount` | `U512` |

use crate::sample::Sample;
use crate::test_data::auction::commons::{self};
use crate::test_data::commons::{prepend_label, sample_executables};
use casper_execution_engine::core::engine_state::ExecutableDeployItem;
use casper_types::{runtime_args, AsymmetricType, PublicKey, RuntimeArgs, U512};

const ENTRY_POINT_NAME: &str = "redelegate";

#[derive(Clone, Debug)]
struct Redelegate {
    delegator: PublicKey,
    validator: PublicKey,
    new_validator: PublicKey,
    amount: U512,
}

impl Redelegate {
    fn new(
        delegator: PublicKey,
        validator: PublicKey,
        new_validator: PublicKey,
        amount: U512,
    ) -> Self {
        Redelegate {
            delegator,
            validator,
            new_validator,
            amount,
        }
    }
}

impl From<Redelegate> for RuntimeArgs {
    fn from(d: Redelegate) -> Self {
        let mut ra = RuntimeArgs::new();
        ra.insert("delegator", d.delegator).unwrap();
        ra.insert("validator", d.validator).unwrap();
        ra.insert("new_validator", d.new_validator).unwrap();
        ra.insert("amount", d.amount).unwrap();
        ra
    }
}

fn invalid_redelegation() -> Vec<Sample<ExecutableDeployItem>> {
    let delegator: PublicKey = PublicKey::ed25519_from_bytes([1u8; 32]).unwrap();
    let old_validator: PublicKey = PublicKey::ed25519_from_bytes([3u8; 32]).unwrap();
    let new_validator: PublicKey = PublicKey::ed25519_from_bytes([6u8; 32]).unwrap();
    let amount = U512::from(100000000u64);

    let valid_args = runtime_args! {
        "delegator" => delegator.clone(),
        "validator" => old_validator.clone(),
        "new_validator" => new_validator.clone(),
        "amount" => amount,
    };

    let invalid_args_samples = {
        let missing_required_amount = runtime_args! {
            "delegator" => delegator.clone(),
            "validator" => old_validator.clone(),
            "new_validator" => new_validator.clone(),
        };

        let missing_required_delegator = runtime_args! {
            "validator" => old_validator.clone(),
            "new_validator" => new_validator.clone(),
            "amount" => amount,
        };

        let missing_required_validator = runtime_args! {
            "delegator" => delegator.clone(),
            "new_validator" => new_validator.clone(),
            "amount" => amount
        };

        let missing_required_new_validator = runtime_args! {
            "delegator" => delegator.clone(),
            "validator" => old_validator.clone(),
            "amount" => amount,
        };

        let invalid_amount_type = runtime_args! {
            "validator" => old_validator,
            "delegator" => delegator,
            "amount" => 100000u32,
            "new_validator" => new_validator,
        };

        // We're setting the "validity bit" to `true`, otherwise such transaction would
        // be rejected by the Ledger Hardware and we don't want that. dApps could be written
        // in such a way that they use similar arguments.
        vec![
            Sample::new("missing_amount", missing_required_amount, true),
            Sample::new("missing_delegator", missing_required_delegator, true),
            Sample::new("missing_validator", missing_required_validator, true),
            Sample::new(
                "missing_new_validator",
                missing_required_new_validator,
                false,
            ),
            Sample::new("invalid_type_amount", invalid_amount_type, true),
        ]
    };

    invalid_args_samples
        .into_iter()
        .flat_map(|sample_ra| {
            let (label, ra, valid) = sample_ra.destructure();
            sample_executables(ENTRY_POINT_NAME, ra, Some(label), valid)
        })
        .chain(
            // Transaction with valid args but invalid entrypoint won't be recognized
            // as proper auction deploy.
            sample_executables(
                "invalid",
                valid_args.clone(),
                Some("invalid_entrypoint".to_string()),
                true, // Even though entrypoint is invalid, it's possible that generic transaction (non-native auction) uses similar set of arguments but changes the entrypoint. In that case, transaction MUSTN'T be invalid b/c it will get rejected by the Ledger.
            ),
        )
        .map(|sample_invalid_executable| prepend_label(sample_invalid_executable, ENTRY_POINT_NAME))
        .collect()
}

// Creates vector of sample `Redelegate` objects.
// Each object in the output vector will have slightly different `amount` field
// so that we cover all edge cases of the `U512` type.
fn sample_redelegations() -> Vec<Redelegate> {
    let amount_min = U512::from(0u8);
    let amount_mid = U512::from(100000000);
    let amount_max = U512::MAX;
    let amounts = vec![amount_min, amount_mid, amount_max];

    let delegator: PublicKey = PublicKey::ed25519_from_bytes([1u8; 32]).unwrap();
    let validator: PublicKey = PublicKey::ed25519_from_bytes([3u8; 32]).unwrap();
    let new_validator: PublicKey = PublicKey::ed25519_from_bytes([6u8; 32]).unwrap();

    amounts
        .into_iter()
        .map(|amount| {
            Redelegate::new(
                delegator.clone(),
                validator.clone(),
                new_validator.clone(),
                amount,
            )
        })
        .collect()
}

pub(crate) fn valid() -> Vec<Sample<ExecutableDeployItem>> {
    let delegate_rargs = sample_redelegations().into_iter().map(Into::into).collect();

    commons::valid(ENTRY_POINT_NAME, delegate_rargs)
}

pub(crate) fn invalid() -> Vec<Sample<ExecutableDeployItem>> {
    invalid_redelegation()
}

mod tests {
    #[test]
    fn redelegate_expected_args() {
        let mut rng = crate::TestRng::new();

        let valid_sample = super::valid();

        fn assertion(args: &casper_types::RuntimeArgs) -> bool {
            args.get("amount").is_some()
                && args.get("delegator").is_some()
                && args.get("validator").is_some()
                && args.get("new_validator").is_some()
        }

        valid_sample.into_iter().for_each(|sample| {
            let (_label, item, _valid) = sample.destructure();
            assert!(
                assertion(item.args()),
                "{:?} did not contain all expected arguments for redelegate deploy",
                item
            )
        });
    }
}
