use hex::decode;
use raycrypt::aeads::aegis256::{decrypt, encrypt};
use serde_json::{from_str, Value};
use std::fs;

#[test]
fn test_encrypt() {
    let key = decode("1001000000000000000000000000000000000000000000000000000000000000").unwrap();
    let nonce = decode("1000020000000000000000000000000000000000000000000000000000000000").unwrap();
    let ad = decode("").unwrap();
    let msg = decode("00000000000000000000000000000000").unwrap();
    let expected_output =
        decode("754fc3d8c973246dcc6d741412a4b2363fe91994768b332ed7f570a19ec5896e").unwrap();

    let output = encrypt::<16>(&key, &msg, &nonce, &ad);

    assert_eq!(output, expected_output);
}

#[test]
fn test_aegis256_wycheproof() {
    let raw = fs::read_to_string("tests/vectors/aegis256.json").unwrap();
    let data: Value = from_str(&raw).unwrap();

    let tests = data["testGroups"][0]["tests"].as_array().unwrap();

    for test in tests {
        let key = hex::decode(test["key"].as_str().unwrap()).unwrap();
        let nonce = hex::decode(test["iv"].as_str().unwrap()).unwrap();
        let aad = hex::decode(test["aad"].as_str().unwrap()).unwrap();
        let pt = hex::decode(test["msg"].as_str().unwrap()).unwrap();

        let ciphertext = hex::decode(test["ct"].as_str().unwrap()).unwrap();
        let tag = hex::decode(test["tag"].as_str().unwrap()).unwrap();

        let expected = [ciphertext.clone(), tag].concat();

        let output = encrypt::<16>(&key, &pt, &nonce, &aad);
        if test["result"].as_str().unwrap() == "valid" {
            assert_eq!(output, expected);

            let decrypted = decrypt::<16>(&key, &output, &nonce, &aad);
            assert_eq!(decrypted.unwrap(), pt);
        } else {
            assert_ne!(output, expected);
        }
    }
}
