"""Comprehensive tests for the SIMI Python bindings."""

import pytest
import simi


# ─── Levenshtein ───────────────────────────────────────────────────

def test_levenshtein_distance():
    assert simi.levenshtein_distance("kitten", "sitting") == 3
    assert simi.levenshtein_distance("hello", "hello") == 0
    assert simi.levenshtein_distance("", "abc") == 3
    assert simi.levenshtein_distance("abc", "") == 3
    assert simi.levenshtein_distance("", "") == 0

def test_levenshtein_similarity():
    assert simi.levenshtein_similarity("hello", "hello") == 1.0
    assert simi.levenshtein_similarity("abc", "xyz") == 0.0
    s = simi.levenshtein_similarity("kitten", "sitting")
    assert 0.5 < s < 0.65

def test_levenshtein_symmetry():
    a = simi.levenshtein_similarity("ab", "ba")
    b = simi.levenshtein_similarity("ba", "ab")
    assert a == b


# ─── Jaro-Winkler ──────────────────────────────────────────────────

def test_jaro_winkler():
    assert simi.jaro_winkler_similarity("hello", "hello") == 1.0
    s = simi.jaro_winkler_similarity("MARTHA", "MARHTA")
    assert 0.95 < s < 0.97
    s = simi.jaro_winkler_similarity("abc", "xyz")
    assert s < 0.2

def test_jaro_winkler_symmetry():
    a = simi.jaro_winkler_similarity("hello", "hallo")
    b = simi.jaro_winkler_similarity("hallo", "hello")
    assert abs(a - b) < 1e-10


# ─── Hamming ───────────────────────────────────────────────────────

def test_hamming_distance():
    assert simi.hamming_distance("karolin", "kathrin") == 3
    assert simi.hamming_distance("hello", "hello") == 0
    assert simi.hamming_distance("abc", "xyz") == 3

def test_hamming_similarity():
    assert simi.hamming_similarity("hello", "hello") == 1.0
    assert simi.hamming_similarity("abc", "xyz") == 0.0
    s = simi.hamming_similarity("karolin", "kathrin")
    assert abs(s - 0.57142) < 0.001

def test_hamming_unequal_length():
    with pytest.raises(ValueError):
        simi.hamming_distance("abc", "abcd")
    with pytest.raises(ValueError):
        simi.hamming_similarity("abc", "abcd")


# ─── Jaccard ───────────────────────────────────────────────────────

def test_jaccard_similarity():
    s = simi.jaccard_similarity("hello", "hello", 2)
    assert s == 1.0
    s = simi.jaccard_similarity("hello", "hallo", 2)
    assert 0.3 < s < 0.7

def test_jaccard_bigram():
    assert simi.jaccard_bigram_similarity("hello", "hello") == 1.0
    s = simi.jaccard_bigram_similarity("abc", "xyz")
    assert 0.0 <= s <= 1.0

def test_jaccard_trigram():
    assert simi.jaccard_trigram_similarity("hello", "hello") == 1.0
    s = simi.jaccard_trigram_similarity("hello", "hallo")
    assert 0.0 < s < 1.0

def test_jaccard_word():
    assert simi.jaccard_word_similarity("a b c", "a b c") == 1.0
    s = simi.jaccard_word_similarity("the quick brown fox", "the quick lazy dog")
    assert abs(s - 0.333) < 0.01


# ─── MinHash ───────────────────────────────────────────────────────

def test_minhash_signature():
    sig = simi.minhash_signature("hello world", 3, 128)
    assert len(sig) == 128
    assert all(isinstance(v, int) for v in sig)

def test_minhash_similarity():
    s = simi.minhash_similarity("hello world", "hello world", 3, 128)
    assert s > 0.9
    s = simi.minhash_similarity("hello world", "totally different text", 3, 128)
    assert 0.0 <= s <= 1.0

def test_minhash_similarity_default():
    s = simi.minhash_similarity_default("hello world", "hello world")
    assert s > 0.9

def test_minhash_symmetry():
    a = simi.minhash_similarity_default("hello world", "hello there")
    b = simi.minhash_similarity_default("hello there", "hello world")
    assert a == b


# ─── SimHash ───────────────────────────────────────────────────────

def test_simhash_fingerprint():
    fp = simi.simhash_fingerprint("hello world", 4)
    assert isinstance(fp, int)
    fp2 = simi.simhash_fingerprint("hello world", 4)
    assert fp == fp2  # deterministic

def test_simhash_fingerprint_default():
    fp = simi.simhash_fingerprint_default("the quick brown fox")
    assert isinstance(fp, int)

def test_simhash_similarity():
    s = simi.simhash_similarity("hello world", "hello world", 4)
    assert s == 1.0

def test_simhash_similarity_default():
    s = simi.simhash_similarity_default("hello world", "hello world")
    assert s > 0.9
    s = simi.simhash_similarity_default("the quick brown fox", "lorem ipsum dolor sit")
    assert 0.0 <= s <= 1.0

def test_simhash_deterministic():
    a = simi.simhash_fingerprint_default("hello")
    b = simi.simhash_fingerprint_default("hello")
    assert a == b


# ─── BM25 ──────────────────────────────────────────────────────────

def test_bm25_similarity():
    assert simi.bm25_similarity("the quick brown fox", "the quick brown fox") == 1.0
    s = simi.bm25_similarity("the quick brown fox", "the quick blue fox")
    assert 0.3 < s < 0.95

def test_bm25_different():
    s = simi.bm25_similarity("hello", "world")
    assert 0.0 <= s <= 1.0


# ─── TF-IDF ────────────────────────────────────────────────────────

def test_tfidf_similarity():
    assert simi.tfidf_similarity("the quick brown fox", "the quick brown fox") == 1.0
    s = simi.tfidf_similarity("the quick brown fox", "the quick blue fox")
    assert 0.3 < s < 0.95

def test_tfidf_different():
    s = simi.tfidf_similarity("abc", "xyz")
    assert s == 0.0

def test_tfidf_symmetry():
    a = simi.tfidf_similarity("the quick brown fox", "the quick red fox")
    b = simi.tfidf_similarity("the quick red fox", "the quick brown fox")
    assert a == b


# ─── Preprocessor ──────────────────────────────────────────────────

def test_clean_text():
    assert simi.clean_text("  Hello   World!  ") == "hello world!"
    assert simi.clean_text("") == ""

def test_clean_text_stopwords():
    result = simi.clean_text_stopwords("the quick brown fox jumps over the lazy dog")
    assert "the" not in result
    assert "quick" in result
    assert "fox" in result

def test_preprocessor_builder():
    pre = simi.Preprocessor() \
        .with_lowercase(True) \
        .with_collapse_whitespace(True) \
        .with_trim(True)
    assert pre.process("  Hello   World  ") == "hello world"

def test_preprocessor_no_lowercase():
    pre = simi.Preprocessor().with_lowercase(False)
    assert pre.process("Hello WORLD") == "Hello World"

def test_preprocessor_custom_stopwords():
    pre = simi.Preprocessor() \
        .with_remove_stopwords(True) \
        .with_stopwords(["hello", "world"])
    result = pre.process("hello wonderful world")
    assert result == "wonderful"

def test_preprocessor_max_length():
    pre = simi.Preprocessor().with_max_length(5)
    assert pre.process("hello world") == "hello"

def test_preprocessor_unicode():
    pre = simi.Preprocessor().with_normalize_unicode(True)
    # e + combining acute -> NFC e-acute
    result = pre.process("\u{0065}\u{0301}")
    assert result == "\u{00e9}"


# ─── SimiFlow router ─────────────────────────────────────────────

def test_flow_tier_1_match():
    b = simi.SimiFlow() \
        .tier_1("jaro_winkler", "gt", 0.95, "lt", 0.10)
    result = b.compare("MARTHA", "MARHTA")
    assert result["tier"] == 1
    assert result["score"] > 0.95

def test_flow_tier_1_mismatch():
    b = simi.SimiFlow() \
        .tier_1("levenshtein", "gt", 0.95, "lt", 0.10)
    result = b.compare("abc", "xyz")
    assert result["tier"] == 1
    assert result["score"] < 0.01

def test_flow_tier_2():
    b = simi.SimiFlow() \
        .tier_1("levenshtein", "gt", 0.95, "lt", 0.05) \
        .tier_2("bm25", "between", 0.30, 0.95)
    result = b.compare("the quick brown fox", "the quick red fox")
    assert result["tier"] == 2

def test_flow_preprocessing():
    b = simi.SimiFlow() \
        .preprocess(True) \
        .tier_1("levenshtein", "gt", 0.95, "lt", 0.10)
    result = b.compare("  Hello   World  ", "hello world")
    assert result["score"] == 1.0

def test_flow_result_keys():
    b = simi.SimiFlow() \
        .tier_1("jaro_winkler", "gt", 0.95, "lt", 0.10)
    result = b.compare("a", "b")
    assert "score" in result
    assert "tier" in result
    assert "algorithm" in result
    assert "fallback_called" in result
    assert "fallback_data" in result

def test_flow_invalid_algo():
    b = simi.SimiFlow()
    with pytest.raises(ValueError):
        b.tier_1("nonexistent", "gt", 0.90, "lt", 0.10)

def test_flow_invalid_threshold():
    b = simi.SimiFlow()
    with pytest.raises(ValueError):
        b.tier_1("jaro_winkler", "invalid", 0.90, "lt", 0.10)

def test_flow_compare_with_intent_names():
    sf = simi.SimiFlow()
    result = sf.compare_with_intent("names", "MARTHA", "MARHTA")
    assert result["algorithm"] == "jaro_winkler"
    assert result["tier"] == 0
    assert result["score"] > 0.95

def test_flow_compare_with_intent_typos():
    sf = simi.SimiFlow()
    result = sf.compare_with_intent("typos", "kitten", "sitting")
    assert result["algorithm"] == "levenshtein"

def test_flow_compare_with_intent_documents():
    sf = simi.SimiFlow()
    result = sf.compare_with_intent("documents", "the quick brown fox", "the quick brown fox")
    assert result["algorithm"] == "bm25"

def test_flow_compare_with_intent_deduplication():
    sf = simi.SimiFlow()
    result = sf.compare_with_intent("dedup", "hello world", "hello world")
    assert result["algorithm"] == "simhash"

def test_flow_compare_with_intent_auto():
    sf = simi.SimiFlow()
    result = sf.compare_with_intent("auto", "abc", "abc")
    assert result["algorithm"] in ("hamming",)  # equal short → Hamming

def test_flow_compare_with_intent_invalid():
    sf = simi.SimiFlow()
    with pytest.raises(ValueError):
        sf.compare_with_intent("nonexistent", "a", "b")


# ─── Edge cases ────────────────────────────────────────────────────

def test_empty_strings():
    assert simi.levenshtein_similarity("", "") == 1.0
    assert simi.jaro_winkler_similarity("", "") == 1.0

def test_empty_vs_nonempty():
    assert simi.levenshtein_similarity("", "hello") == 0.0
    assert simi.tfidf_similarity("", "hello") == 0.0

def test_unicode():
    s = simi.levenshtein_similarity("cafe", "cafe")
    assert s == 1.0

def test_long_strings():
    a = "a" * 1000
    b = "a" * 999 + "b"
    s = simi.levenshtein_similarity(a, b)
    assert s > 0.99

def test_simhash_different_long():
    a = "the quick brown fox jumps over the lazy dog on a sunny afternoon by the river bank with green trees and blue skies"
    b = "lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod tempor incididunt ut labore et dolore magna aliqua"
    s = simi.simhash_similarity_default(a, b)
    assert 0.0 <= s <= 1.0
