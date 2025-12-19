use reticulum_rs::{FullIdentity, Reticulum, MTU};
use rand_core::OsRng;

fn main() {
    let _reticulum = Reticulum::new();
    let identity = FullIdentity::generate(&mut OsRng);
    
    println!("Reticulum initialized (MTU: {} bytes)", MTU);
    println!("Identity: {}", hex::encode(identity.identity().address_hash().as_bytes()));
}
