# SIMI Node.js Bindings

Node.js bindings for the SIMI similarity toolkit, powered by napi-rs.

## Installation

```bash
npm install @siktec-lab/simi
```

## Usage

```javascript
const simi = require('@siktec-lab/simi');

// Levenshtein similarity
console.log(simi.levenshtein_similarity('kitten', 'sitting'));
// → 0.571

// Jaro-Winkler similarity (great for names)
console.log(simi.jaro_winkler_similarity('MARTHA', 'MARHTA'));
// → ~0.961

// Text preprocessing
console.log(simi.clean_text('  Hello   World!  '));
// → 'hello world!'
```
