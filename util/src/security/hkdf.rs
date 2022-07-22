pub fn expand_extract<S, I>(salt: S, len: usize, input: I) -> Vec<u8>
where
    S: AsRef<[u8; 32]>,
    I: AsRef<[u8]>
{
    use crypto::{
        hkdf,
        sha2::Sha256
    };

    let mut prk = [0u8; 256 / 8];
    let mut output = vec![0u8; len];

    hkdf::hkdf_extract(Sha256::new(), salt.as_ref(), input.as_ref(), prk.as_mut_slice());
    hkdf::hkdf_expand(Sha256::new(), prk.as_slice(), &[], output.as_mut_slice());

    output
}
