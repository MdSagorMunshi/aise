use aise_core::sponge;
use aise_core::ph;
use aise_core::commit;
use aise_core::ratchet;

#[test]
fn test_empty_string_hash_consistency() {
    let out = sponge::aise_hash(&[], 64);
    assert_eq!(out.len(), 64);
}

#[test]
fn test_xof_expansion() {
    let out = sponge::aise_xof(&[], 4096);
    let non_zero = out.iter().filter(|&&x| x != 0).count();
    assert!(non_zero > 4000);
}

#[test]
fn test_aise_ph_tunability() {
    let pwd = b"password";
    let salt = b"salt";
    let hash1 = ph::aise_ph(pwd, salt, 1, 64);
    let hash2 = ph::aise_ph(pwd, salt, 2, 64);
    assert_ne!(hash1, hash2);
}

#[test]
fn test_aise_commit_open() {
    let val = b"secret_value";
    let rand = b"random_entropy";
    let c = commit::aise_commit(val, rand);
    assert!(commit::aise_open(&c, val, rand));
    assert!(!commit::aise_open(&c, b"wrong_value", rand));
}

#[test]
fn test_aise_ratchet() {
    let init_secret = b"initial";
    let state0 = ratchet::aise_ratchet_init(init_secret);
    let (state1, key0) = ratchet::aise_ratchet_step(state0);
    let (state2, key1) = ratchet::aise_ratchet_step(state1);
    
    assert_ne!(key0, key1);
    assert_ne!(state2, key0);
}
