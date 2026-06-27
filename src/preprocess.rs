//! String preprocessors: zero-copy string cleanup that normalizes unicode,
//! handles whitespace, and removes stop-words before the math happens.

use unicode_normalization::UnicodeNormalization;

/// Configuration for text preprocessing.
#[derive(Clone, Debug)]
pub struct Preprocessor {
    /// Whether to normalize unicode to NFC form.
    pub normalize_unicode: bool,
    /// Whether to collapse whitespace to single spaces.
    pub collapse_whitespace: bool,
    /// Whether to trim leading/trailing whitespace.
    pub trim: bool,
    /// Whether to convert to lowercase.
    pub to_lowercase: bool,
    /// Whether to remove stop words.
    pub remove_stopwords: bool,
    /// Custom stop words (if empty, uses built-in default set).
    pub stopwords: Vec<String>,
    /// Maximum string length to process (0 = unlimited).
    pub max_length: usize,
}

impl Default for Preprocessor {
    fn default() -> Self {
        Self {
            normalize_unicode: true,
            collapse_whitespace: true,
            trim: true,
            to_lowercase: true,
            remove_stopwords: false,
            stopwords: Vec::new(),
            max_length: 0,
        }
    }
}

impl Preprocessor {
    /// Create a new preprocessor with default settings.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set unicode normalization.
    #[inline]
    pub fn with_normalize_unicode(mut self, v: bool) -> Self {
        self.normalize_unicode = v;
        self
    }

    /// Set whitespace collapsing.
    #[inline]
    pub fn with_collapse_whitespace(mut self, v: bool) -> Self {
        self.collapse_whitespace = v;
        self
    }

    /// Set trimming.
    #[inline]
    pub fn with_trim(mut self, v: bool) -> Self {
        self.trim = v;
        self
    }

    /// Set lowercase conversion.
    #[inline]
    pub fn with_lowercase(mut self, v: bool) -> Self {
        self.to_lowercase = v;
        self
    }

    /// Set stopword removal.
    #[inline]
    pub fn with_remove_stopwords(mut self, v: bool) -> Self {
        self.remove_stopwords = v;
        self
    }

    /// Set custom stop words.
    #[inline]
    pub fn with_stopwords(mut self, words: Vec<String>) -> Self {
        self.stopwords = words;
        self
    }

    /// Set max input length.
    #[inline]
    pub fn with_max_length(mut self, max: usize) -> Self {
        self.max_length = max;
        self
    }

    /// Preprocess a text string.
    ///
    /// Returns the processed string, or an error if processing fails.
    #[inline]
    pub fn process(&self, text: &str) -> String {
        let mut s = text.to_string();

        // Unicode normalization (NFC)
        if self.normalize_unicode {
            s = s.nfc().collect();
        }

        // Lowercase
        if self.to_lowercase {
            s = s.to_lowercase();
        }

        // Collapse whitespace
        if self.collapse_whitespace {
            let mut result = String::with_capacity(s.len());
            let mut prev_was_space = false;
            for c in s.chars() {
                if c.is_whitespace() {
                    if !prev_was_space {
                        result.push(' ');
                        prev_was_space = true;
                    }
                } else {
                    result.push(c);
                    prev_was_space = false;
                }
            }
            s = result;
        }

        // Trim
        if self.trim {
            s = s.trim().to_string();
        }

        // Stopword removal
        if self.remove_stopwords {
            let result: Vec<&str> = s
                .split_whitespace()
                .filter(|word| !is_stopword(word, &self.stopwords))
                .collect();
            s = result.join(" ");
        }

        // Max length
        if self.max_length > 0 && s.len() > self.max_length {
            s.truncate(self.max_length);
        }

        s
    }
}

// Default English stopword list (common words that carry little semantic weight)
const DEFAULT_STOPWORDS: &[&str] = &[
    "a",
    "an",
    "the",
    "and",
    "or",
    "but",
    "if",
    "because",
    "as",
    "until",
    "while",
    "of",
    "at",
    "by",
    "for",
    "with",
    "about",
    "between",
    "into",
    "through",
    "during",
    "before",
    "after",
    "above",
    "below",
    "to",
    "from",
    "up",
    "down",
    "in",
    "out",
    "on",
    "off",
    "over",
    "under",
    "again",
    "further",
    "then",
    "once",
    "here",
    "there",
    "when",
    "where",
    "why",
    "how",
    "all",
    "each",
    "every",
    "both",
    "few",
    "more",
    "most",
    "other",
    "some",
    "such",
    "no",
    "nor",
    "not",
    "only",
    "own",
    "same",
    "so",
    "than",
    "too",
    "very",
    "just",
    "also",
    "am",
    "is",
    "are",
    "was",
    "were",
    "be",
    "been",
    "being",
    "have",
    "has",
    "had",
    "having",
    "do",
    "does",
    "did",
    "doing",
    "would",
    "could",
    "should",
    "might",
    "must",
    "shall",
    "can",
    "will",
    "may",
    "need",
    "dare",
    "ought",
    "used",
    "this",
    "that",
    "these",
    "those",
    "i",
    "me",
    "my",
    "myself",
    "we",
    "our",
    "ours",
    "ourselves",
    "you",
    "your",
    "yours",
    "yourself",
    "yourselves",
    "he",
    "him",
    "his",
    "himself",
    "she",
    "her",
    "hers",
    "herself",
    "it",
    "its",
    "itself",
    "they",
    "them",
    "their",
    "theirs",
    "themselves",
    "what",
    "which",
    "who",
    "whom",
    "whose",
    "any",
    "anyone",
    "anything",
    "anybody",
    "everyone",
    "everything",
    "everybody",
    "someone",
    "something",
    "somebody",
    "nobody",
    "nothing",
    "none",
    "neither",
    "one",
    "two",
    "three",
    "get",
    "got",
    "getting",
    "make",
    "made",
    "making",
    "take",
    "took",
    "taking",
    "let",
    "lets",
    "letting",
    "like",
    "liked",
    "likes",
    "really",
    "actually",
    "basically",
    "probably",
    "maybe",
    "perhaps",
    "quite",
    "rather",
    "pretty",
    "almost",
    "nearly",
    "hardly",
    "scarcely",
    "barely",
    "already",
    "yet",
    "still",
];

/// Check if a word is a stopword, using custom list or the built-in default.
#[inline]
fn is_stopword(word: &str, custom: &[String]) -> bool {
    if custom.is_empty() {
        DEFAULT_STOPWORDS.contains(&word)
    } else {
        custom.iter().any(|w| w == word)
    }
}

/// Convenience: preprocess text with default settings.
#[inline]
pub fn clean(text: &str) -> String {
    Preprocessor::default().process(text)
}

/// Convenience: preprocess and remove stopwords.
#[inline]
pub fn clean_with_stopwords(text: &str) -> String {
    Preprocessor::default()
        .with_remove_stopwords(true)
        .process(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_cleanup() {
        let cleaned = clean("  Hello   World!  ");
        assert_eq!(cleaned, "hello world!");
    }

    #[test]
    fn unicode_normalization() {
        // Pre-composed vs decomposed é
        let s = clean("\u{0065}\u{0301}"); // e + combining acute
        assert_eq!(s, "\u{00e9}"); // NFC should collapse to é
    }

    #[test]
    fn stopword_removal() {
        let result = clean_with_stopwords("the quick brown fox jumps over the lazy dog");
        // "the" and "over" should be removed
        assert!(!result.contains("the "));
        assert!(result.contains("quick"));
        assert!(result.contains("fox"));
        assert!(result.contains("jumps"));
        assert!(result.contains("lazy"));
        assert!(result.contains("dog"));
    }

    #[test]
    fn lowercase() {
        let pre = Preprocessor::default().with_lowercase(true);
        assert_eq!(pre.process("Hello WORLD"), "hello world");
    }

    #[test]
    fn no_lowercase() {
        let pre = Preprocessor::default().with_lowercase(false);
        assert_eq!(pre.process("Hello WORLD"), "Hello WORLD");
    }

    #[test]
    fn custom_stopwords() {
        let words = vec!["hello".to_string(), "world".to_string()];
        let pre = Preprocessor::default()
            .with_remove_stopwords(true)
            .with_stopwords(words);
        let result = pre.process("hello wonderful world");
        assert_eq!(result, "wonderful");
    }

    #[test]
    fn trim_input() {
        let pre = Preprocessor::default().with_trim(true);
        assert_eq!(pre.process("  spaced  "), "spaced");
    }

    #[test]
    fn max_length() {
        let pre = Preprocessor::default().with_max_length(5);
        assert_eq!(pre.process("hello world"), "hello");
    }

    #[test]
    fn builder_pattern() {
        let pre = Preprocessor::new()
            .with_normalize_unicode(true)
            .with_collapse_whitespace(true)
            .with_trim(true)
            .with_lowercase(true);
        assert_eq!(pre.process("  Hello   World  "), "hello world");
    }
}
