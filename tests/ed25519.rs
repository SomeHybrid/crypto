use raycrypt::ecc::ed25519::VerifyingKey;
use serde_json::{from_str, Value};
use std::fs;

#[test]
fn test_ed25519_wycheproof() {
    let raw = fs::read_to_string("tests/vectors/ed25519.json").unwrap();
    let data: Value = from_str(&raw).unwrap();

    let testgroups = data["testGroups"].as_array().unwrap();
    for testgroup in testgroups {
        let tests = testgroup["tests"].as_array().unwrap();
        let pkey = hex::decode(
            testgroup["publicKey"].as_object().unwrap()["pk"]
                .as_str()
                .unwrap(),
        )
        .unwrap();

        let verifier = VerifyingKey::from(&pkey).unwrap();

        for test in tests {
            let test = test.as_object().unwrap();
            let msg = hex::decode(test["msg"].as_str().unwrap()).unwrap();
            let signature = hex::decode(test["sig"].as_str().unwrap()).unwrap();
            println!("{}", test["tcId"].as_u64().unwrap());
            if test["result"].as_str().unwrap() == "valid" {
                assert!(verifier.verify(&msg, &signature));
            } else {
                assert!(!verifier.verify(&msg, &signature));
            }
        }
    }
}
