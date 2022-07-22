pub fn md5<T: AsRef<[u8]>>(input: T) -> [u8; 16] {
    md5::compute(input).0
}

pub fn sha256<T: AsRef<[u8]>>(first: T, second: T) -> [u8; 32] {
    use crypto::digest::Digest;

    let mut hasher = crypto::sha2::Sha256::new();
    hasher.input(first.as_ref());
    hasher.input(second.as_ref());

    let mut output = [0u8; 32];
    hasher.result(output.as_mut_slice());

    output
}