//! MinHash — probabilistic data structure for estimating Jaccard similarity
//! between large documents. Uses k hash functions to create compact
//! fingerprints.
//!
//! ## Performance
//! O(k·n) time where k = number of hash functions, n = shingle count.
//! Fingerprint size is O(k) memory per document.

use std::collections::HashSet;

const DEFAULT_NUM_HASHES: usize = 128;
const DEFAULT_SHINGLE_SIZE: usize = 3;

/// A MinHash fingerprint for estimating Jaccard similarity between documents.
///
/// The fingerprint is a vector of minimum hash values computed over the
/// document's shingles. Two documents with similar content will have similar
/// fingerprints.
#[derive(Clone, Debug)]
pub struct MinHash {
    /// The computed minimum hash values (the fingerprint).
    pub signatures: Vec<u64>,
    /// Number of hash functions used.
    pub num_hashes: usize,
}

/// Pre-computed random coefficients for the universal hash family.
///
/// Uses (a * x + b) mod P with randomly chosen a, b values to create
/// k independent hash functions.
fn generate_hash_coeffs(num_hashes: usize) -> (Vec<u64>, Vec<u64>) {
    // Use deterministic seeds derived from a fixed base; in production
    // you'd persist these across MinHash runs for consistency.
    let seed: u64 = 0x9e3779b97f4a7c15;
    let mut a_coeffs = Vec::with_capacity(num_hashes);
    let mut b_coeffs = Vec::with_capacity(num_hashes);

    for i in 0..num_hashes {
        // Simple LCG-style coefficient generation
        let a = seed.wrapping_mul(i as u64 + 1).wrapping_add(0xABCD);
        let b = seed.wrapping_mul(i as u64 + 1).wrapping_add(0x1234);
        a_coeffs.push(a | 1); // ensure odd (coprime with power of 2 modulus)
        b_coeffs.push(b);
    }

    (a_coeffs, b_coeffs)
}

/// Compute a MinHash signature for a string.
///
/// # Arguments
/// * `text` - The input text.
/// * `shingle_size` - Size of character-level shingles (default: 3).
/// * `num_hashes` - Number of hash functions (default: 128).
///
/// Returns a `MinHash` fingerprint.
#[inline]
pub fn signature(text: &str, shingle_size: usize, num_hashes: usize) -> MinHash {
    // Generate shingles
    let chars: Vec<char> = text.chars().collect();
    let mut shingles = HashSet::new();

    if chars.len() < shingle_size {
        shingles.insert(text.to_string());
    } else {
        for w in chars.windows(shingle_size) {
            shingles.insert(w.iter().collect::<String>());
        }
    }

    let (a_coeffs, b_coeffs) = generate_hash_coeffs(num_hashes);
    // Large prime for universal hashing
    const P: u64 = (1 << 61) - 1;

    let mut signatures = vec![u64::MAX; num_hashes];

    for shingle in &shingles {
        let shingle_hash = fxhash(shingle);

        for i in 0..num_hashes {
            let hash_val = a_coeffs[i]
                .wrapping_mul(shingle_hash)
                .wrapping_add(b_coeffs[i])
                % P;
            if hash_val < signatures[i] {
                signatures[i] = hash_val;
            }
        }
    }

    MinHash {
        signatures,
        num_hashes,
    }
}

/// A simple fast hash for short strings (FNV-1a like).
fn fxhash(s: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// Estimate Jaccard similarity between two MinHash signatures.
///
/// Returns a value in `[0.0, 1.0]`.
#[inline]
pub fn similarity(a: &MinHash, b: &MinHash) -> f64 {
    let min_len = std::cmp::min(a.signatures.len(), b.signatures.len());
    let matches = a.signatures[..min_len]
        .iter()
        .zip(b.signatures[..min_len].iter())
        .filter(|(sa, sb)| sa == sb)
        .count();

    matches as f64 / min_len as f64
}

/// Convenience: shingle + signature + similarity in one call.
#[inline]
pub fn compare(a: &str, b: &str, shingle_size: usize, num_hashes: usize) -> f64 {
    let sig_a = signature(a, shingle_size, num_hashes);
    let sig_b = signature(b, shingle_size, num_hashes);
    similarity(&sig_a, &sig_b)
}

/// Default convenience: trigram shingles, 128 hashes.
#[inline]
pub fn compare_default(a: &str, b: &str) -> f64 {
    compare(a, b, DEFAULT_SHINGLE_SIZE, DEFAULT_NUM_HASHES)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_documents() {
        let sig_a = signature("hello world this is a test", 3, 128);
        let sig_b = signature("hello world this is a test", 3, 128);
        let s = similarity(&sig_a, &sig_b);
        assert!((s - 1.0).abs() < 0.01, "identical should be ~1.0, got {s}");
    }

    #[test]
    fn similar_documents() {
        let s = compare_default(
            "the quick brown fox jumps over the lazy dog",
            "the quick brown fox jumps over the lazy cat",
        );
        assert!(s > 0.5, "expected high similarity, got {s}");
    }

    #[test]
    fn different_documents() {
        let s = compare_default(
            "hello world this is a test",
            "completely unrelated content nothing similar",
        );
        assert!(s < 0.3, "expected low similarity, got {s}");
    }

    #[test]
    fn symmetric() {
        let a = compare_default("abc def ghi", "abc xyz");
        let b = compare_default("abc xyz", "abc def ghi");
        assert!((a - b).abs() < f64::EPSILON);
    }

    #[test]
    fn consistent_signature_length() {
        let sig = signature("test", 3, 128);
        assert_eq!(sig.signatures.len(), 128);
        assert_eq!(sig.num_hashes, 128);
    }
}
