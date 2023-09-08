use encryption::poly1305::Poly1305;

#[test]
fn test_poly1305_1() {
    let key = [
        0x85, 0xd6, 0xbe, 0x78, 0x57, 0x55, 0x6d, 0x33, 0x7f, 0x44, 0x52, 0xfe, 0x42, 0xd5, 0x06,
        0xa8, 0x01, 0x03, 0x80, 0x8a, 0xfb, 0x0d, 0xb2, 0xfd, 0x4a, 0xbf, 0xf6, 0xaf, 0x41, 0x49,
        0xf5, 0x1b,
    ];
    let msg = b"Cryptographic Forum Research Group";
    let tag = [
        0xa8, 0x06, 0x1d, 0xc1, 0x30, 0x51, 0x36, 0xc6, 0xc2, 0x2b, 0x8b, 0xaf, 0x0c, 0x01, 0x27,
        0xa9,
    ];

    let mut p = Poly1305::new(key.to_vec());
    p.update_unpadded(msg);
    assert_eq!(p.tag(), &tag);
}

#[test]
fn test_poly1305_2() {
    let key = [
        0xee, 0xa6, 0xa7, 0x25, 0x1c, 0x1e, 0x72, 0x91, 0x6d, 0x11, 0xc2, 0xcb, 0x21, 0x4d, 0x3c,
        0x25, 0x25, 0x39, 0x12, 0x1d, 0x8e, 0x23, 0x4e, 0x65, 0x2d, 0x65, 0x1f, 0xa4, 0xc8, 0xcf,
        0xf8, 0x80,
    ];

    let msg = [
        0x8e, 0x99, 0x3b, 0x9f, 0x48, 0x68, 0x12, 0x73, 0xc2, 0x96, 0x50, 0xba, 0x32, 0xfc, 0x76,
        0xce, 0x48, 0x33, 0x2e, 0xa7, 0x16, 0x4d, 0x96, 0xa4, 0x47, 0x6f, 0xb8, 0xc5, 0x31, 0xa1,
        0x18, 0x6a, 0xc0, 0xdf, 0xc1, 0x7c, 0x98, 0xdc, 0xe8, 0x7b, 0x4d, 0xa7, 0xf0, 0x11, 0xec,
        0x48, 0xc9, 0x72, 0x71, 0xd2, 0xc2, 0x0f, 0x9b, 0x92, 0x8f, 0xe2, 0x27, 0x0d, 0x6f, 0xb8,
        0x63, 0xd5, 0x17, 0x38, 0xb4, 0x8e, 0xee, 0xe3, 0x14, 0xa7, 0xcc, 0x8a, 0xb9, 0x32, 0x16,
        0x45, 0x48, 0xe5, 0x26, 0xae, 0x90, 0x22, 0x43, 0x68, 0x51, 0x7a, 0xcf, 0xea, 0xbd, 0x6b,
        0xb3, 0x73, 0x2b, 0xc0, 0xe9, 0xda, 0x99, 0x83, 0x2b, 0x61, 0xca, 0x01, 0xb6, 0xde, 0x56,
        0x24, 0x4a, 0x9e, 0x88, 0xd5, 0xf9, 0xb3, 0x79, 0x73, 0xf6, 0x22, 0xa4, 0x3d, 0x14, 0xa6,
        0x59, 0x9b, 0x1f, 0x65, 0x4c, 0xb4, 0x5a, 0x74, 0xe3, 0x55, 0xa5,
    ];

    let expected = [
        0xf3, 0xff, 0xc7, 0x70, 0x3f, 0x94, 0x00, 0xe5, 0x2a, 0x7d, 0xfb, 0x4b, 0x3d, 0x33, 0x05,
        0xd9,
    ];

    let mut p = Poly1305::new(key.to_vec());
    p.update_unpadded(&msg);
    assert_eq!(p.tag(), &expected);
}

#[test]
fn donna_test_1() {
    let nacl_key = [
        0xee, 0xa6, 0xa7, 0x25, 0x1c, 0x1e, 0x72, 0x91, 0x6d, 0x11, 0xc2, 0xcb, 0x21, 0x4d, 0x3c,
        0x25, 0x25, 0x39, 0x12, 0x1d, 0x8e, 0x23, 0x4e, 0x65, 0x2d, 0x65, 0x1f, 0xa4, 0xc8, 0xcf,
        0xf8, 0x80,
    ];

    let nacl_msg = [
        0x8e, 0x99, 0x3b, 0x9f, 0x48, 0x68, 0x12, 0x73, 0xc2, 0x96, 0x50, 0xba, 0x32, 0xfc, 0x76,
        0xce, 0x48, 0x33, 0x2e, 0xa7, 0x16, 0x4d, 0x96, 0xa4, 0x47, 0x6f, 0xb8, 0xc5, 0x31, 0xa1,
        0x18, 0x6a, 0xc0, 0xdf, 0xc1, 0x7c, 0x98, 0xdc, 0xe8, 0x7b, 0x4d, 0xa7, 0xf0, 0x11, 0xec,
        0x48, 0xc9, 0x72, 0x71, 0xd2, 0xc2, 0x0f, 0x9b, 0x92, 0x8f, 0xe2, 0x27, 0x0d, 0x6f, 0xb8,
        0x63, 0xd5, 0x17, 0x38, 0xb4, 0x8e, 0xee, 0xe3, 0x14, 0xa7, 0xcc, 0x8a, 0xb9, 0x32, 0x16,
        0x45, 0x48, 0xe5, 0x26, 0xae, 0x90, 0x22, 0x43, 0x68, 0x51, 0x7a, 0xcf, 0xea, 0xbd, 0x6b,
        0xb3, 0x73, 0x2b, 0xc0, 0xe9, 0xda, 0x99, 0x83, 0x2b, 0x61, 0xca, 0x01, 0xb6, 0xde, 0x56,
        0x24, 0x4a, 0x9e, 0x88, 0xd5, 0xf9, 0xb3, 0x79, 0x73, 0xf6, 0x22, 0xa4, 0x3d, 0x14, 0xa6,
        0x59, 0x9b, 0x1f, 0x65, 0x4c, 0xb4, 0x5a, 0x74, 0xe3, 0x55, 0xa5,
    ];

    let nacl_mac = [
        0xf3, 0xff, 0xc7, 0x70, 0x3f, 0x94, 0x00, 0xe5, 0x2a, 0x7d, 0xfb, 0x4b, 0x3d, 0x33, 0x05,
        0xd9,
    ];

    let mut p = Poly1305::new(nacl_key.to_vec());
    p.update_unpadded(&nacl_msg);

    assert_eq!(p.verify(&nacl_mac), true);
}

#[test]
fn donna_test_2() {
    let wrap_key = [
        0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ];

    let wrap_msg = [
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff,
    ];

    let wrap_mac = [
        0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ];

    let mut p2 = Poly1305::new(wrap_key.to_vec());
    p2.update_unpadded(&wrap_msg);

    assert_eq!(p2.verify(&wrap_mac), true);
}
