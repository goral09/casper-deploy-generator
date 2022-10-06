use ledger::ZondaxRepr;
use test_data::{
    delegate_samples, generic_samples, native_transfer_samples, redelegate_samples,
    undelegate_samples,
};
use test_rng::TestRng;

pub mod checksummed_hex;
mod ledger;
mod parser;
mod sample;
mod test_data;
mod test_rng;
mod utils;

fn main() {
    let mut rng = TestRng::new();

    let data: Vec<ZondaxRepr> = undelegate_samples(&mut rng)
        .into_iter()
        .chain(delegate_samples(&mut rng))
        .chain(native_transfer_samples(&mut rng))
        .chain(redelegate_samples(&mut rng))
        .chain(generic_samples(&mut rng))
        .enumerate()
        .map(|(id, sample_deploy)| ledger::deploy_to_json(id, sample_deploy))
        .collect();

    println!("{}", serde_json::to_string_pretty(&data).unwrap());
}
