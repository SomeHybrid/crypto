use raycrypt::{decrypt, encrypt};

#[test]
fn test_encrypt() {
    let key = b"1234567890ABCDEF1234567890ABCDEF".to_vec();
    let msg = b"hello there";
    let encrypted = encrypt(key.clone(), msg);
    let decrypted = decrypt(key, &encrypted);
    assert_eq!(msg.to_vec(), decrypted.unwrap());
}
