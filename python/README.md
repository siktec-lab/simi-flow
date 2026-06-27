# SIMI Python Bindings

Python bindings for the SIMI similarity toolkit, powered by PyO3.

## Installation

```bash
pip install simi-sim
```

## Usage

```python
import simi

# Levenshtein similarity
print(simi.levenshtein_similarity("kitten", "sitting"))
# → 0.571

# Jaro-Winkler similarity (great for names)
print(simi.jaro_winkler_similarity("MARTHA", "MARHTA"))
# → ~0.961

# Text preprocessing
print(simi.clean_text("  Hello   World!  "))
# → "hello world!"
```
