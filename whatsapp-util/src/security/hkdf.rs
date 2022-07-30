pub fn expand_extract<I>(salt: [u8; 32], input: I) -> [u8; 64]
where
    I: AsRef<[u8]>,
{
    use crypto::{hkdf, sha2::Sha256};

    let mut prk = [0u8; 256 / 8];
    let mut output = [0u8; 64];

    hkdf::hkdf_extract(Sha256::new(), &salt, input.as_ref(), prk.as_mut_slice());
    hkdf::hkdf_expand(Sha256::new(), prk.as_slice(), &[], output.as_mut_slice());

    output
}
