"""Tests for the SIMI Python bindings."""

import simi


def test_levenshtein():
    assert simi.levenshtein_similarity("hello", "hello") == 1.0
    score = simi.levenshtein_similarity("kitten", "sitting")
    assert 0.5 < score < 0.65


def test_jaro_winkler():
    score = simi.jaro_winkler_similarity("MARTHA", "MARHTA")
    assert 0.95 < score < 0.97


def test_hamming():
    assert simi.hamming_similarity("hello", "hello") == 1.0
    assert simi.hamming_similarity("abc", "xyz") == 0.0


def test_jaccard():
    score = simi.jaccard_similarity("hello world", "hello there", 2)
    assert 0.0 < score < 1.0


def test_bm25():
    score = simi.bm25_similarity("the quick brown fox", "the quick brown fox")
    assert abs(score - 1.0) < 0.01


def test_tfidf():
    score = simi.tfidf_similarity("the quick brown fox", "the quick blue fox")
    assert 0.3 < score < 1.0


def test_clean_text():
    result = simi.clean_text("  Hello   World!  ")
    assert result == "hello world!"
