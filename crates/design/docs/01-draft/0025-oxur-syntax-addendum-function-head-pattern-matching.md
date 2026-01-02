---
number: 25
title: "Oxur Syntax Addendum: Function Head Pattern Matching"
author: "declaring the"
created: 2026-01-01
updated: 2026-01-01
state: Draft
supersedes: null
superseded-by: null
version: 1.0
---

# Oxur Syntax Addendum: Function Head Pattern Matching

## Multi-Clause Functions with Pattern Matching

**Related**: Oxur v1.0 Design Document

---

## Table of Contents

1. [Overview](#overview)
2. [Motivation](#motivation)
3. [Syntax](#syntax)
4. [Examples](#examples)
5. [Macro Expansion](#macro-expansion)
6. [Design Rationale](#design-rationale)
7. [Comparison with Other Languages](#comparison-with-other-languages)
8. [Implementation Notes](#implementation-notes)
9. [Future Considerations](#future-considerations)

---

## 1. Overview

**NOTE: This addendum explores experimental syntax for multi-clause functions. The syntax presented here represents our latest iteration, achieving near-perfect alignment between explicit `match` and function head matching styles.**

This addendum proposes extending Oxur's `deffn` macro to support **multi-clause function definitions** with pattern matching in function heads, inspired by LFE (Lisp Flavored Erlang), Erlang, ML, and Haskell.

### Key Design Insight

By declaring the function signature once (with parameters and return type), then providing pattern clauses that match against those parameters, we achieve **maximal consistency** between explicit `match` and function head styles.

### Explicit Match Syntax

```clojure
(deffn fibonacci (n:i32) (:> i32)
  (match n
    (0 0)
    (1 1)
    (n (+ (fibonacci (- n 1)) (fibonacci (- n 2))))))
```

### Function Head Matching Syntax

```clojure
(deffn fibonacci (n:i32) (:> i32)
  (0 0)
  (1 1)
  (n (+ (fibonacci (- n 1)) (fibonacci (- n 2)))))
```

**Notice**: The only difference is the presence of `(match n ...)` wrapper! Both forms are valid Oxur. The function head form is syntactic sugar that expands to the explicit match form.

### Why This Works

Since Rust (and therefore Oxur) requires all match arms to return the same type, we only need to declare the return type once. The macro detects whether the body is:

1. An explicit `match` expression
2. A series of pattern clauses (function heads)
3. A regular function body

This makes the macro **simpler** while providing **better developer experience**.

---

## 2. Motivation

### 2.1 Functional Programming Heritage

Languages in the ML family (SML, OCaml, Haskell) and Erlang/LFE use pattern matching in function definitions as a primary idiom. This style:

- **Separates cases visually** - Each clause stands alone
- **Reduces nesting** - No need for nested `match` expressions
- **Mirrors mathematical definitions** - Looks like mathematical piecewise functions
- **Feels natural** - For functional programmers coming from those traditions

### 2.2 Readability and Consistency

The new design brings both styles into perfect alignment:

**Explicit Match:**

```clojure
(deffn length (t) (items:(vec t)) (:> usize)
  (match items
    ((vec::new) 0)
    ((cons _ rest) (+ 1 (length rest)))))
```

**Function Head Matching:**

```clojure
(deffn length (t) (items:(vec t)) (:> usize)
  ((vec::new) 0)
  ((cons _ rest) (+ 1 (length rest))))
```

The **only difference** is `(match items ...)` - everything else is identical! This makes it:

- **Easy to learn** - If you know `match`, you know function heads
- **Easy to convert** - Add/remove `match` wrapper
- **Visually consistent** - Same indentation, same structure
- **No repetition** - Return type declared once

### 2.3 Guards are Natural

Guards fit naturally and identically in both forms:

**Explicit Match:**

```clojure
(deffn classify (n:i32) (:> string)
  (match n
    (n (when (< n 0)) "negative")
    (0 "zero")
    (n (when (> n 0)) "positive")))
```

**Function Head Matching:**

```clojure
(deffn classify (n:i32) (:> string)
  (n (when (< n 0)) "negative")
  (0 "zero")
  (n (when (> n 0)) "positive"))
```

### 2.4 No New Semantics

This is **purely syntactic sugar**. It desugars to standard `match` expressions, so:

- No new runtime behavior
- No new type checking rules
- No performance implications
- Just an alternative syntax for clarity

---

## 3. Syntax

### 3.1 Unified Form

```clojure
(deffn function-name (type-params*) (params*) (:> return-type)
  body)
```

Where `body` can be one of three forms:

**1. Explicit Match:**

```clojure
(match expr
  (pattern body)
  (pattern body)
  ...)
```

**2. Function Head Matching (sugar for match on parameters):**

```clojure
(pattern body)
(pattern body)
...
```

**3. Regular Function Body:**

```clojure
(expr1)
(expr2)
...
```

### 3.2 Detection Logic

The `deffn` macro detects which form is being used:

**Explicit Match** - First element is a `match` form:

```clojure
(deffn factorial (n:i32) (:> i32)
  (match n
    (0 1)
    (n (* n (factorial (- n 1))))))
```

**Function Head Matching** - Body consists of pattern clauses:

```clojure
(deffn factorial (n:i32) (:> i32)
  (0 1)
  (n (* n (factorial (- n 1)))))
```

**Regular Body** - Body is normal expressions:

```clojure
(deffn add (x:i32 y:i32) (:> i32)
  (+ x y))
```

### 3.3 Pattern Clause Structure

A pattern clause consists of:

```clojure
(pattern guard? body*)
```

**Pattern**: Matches Rust's pattern syntax

- Literals: `0`, `1`, `"hello"`
- Bindings: `n`, `x`, `value`
- Wildcards: `_`
- Destructuring: `(some x)`, `(cons first rest)`, `(point x y)`
- Tuples (for multi-param): `(a 0)`, `(x y)`

**Guard**: Optional `(when condition)`

```clojure
(n (when (> n 0)) "positive")
```

**Body**: One or more expressions

```clojure
42
(+ x y)
(do (println! "doing work") result)
```

### 3.4 Single Parameter Functions

When matching a single parameter, patterns match directly:

```clojure
(deffn factorial (n:i32) (:> i32)
  (0 1)
  (n (* n (factorial (- n 1)))))

;; Expands to:
(deffn factorial (n:i32) (:> i32)
  (match n
    (0 1)
    (n (* n (factorial (- n 1))))))
```

### 3.5 Multiple Parameter Functions

When matching multiple parameters, patterns are tuples:

```clojure
(deffn gcd (a:i32 b:i32) (:> i32)
  (a 0) a)
  (a b) (gcd b (mod a b))))

;; Expands to:
(deffn gcd (a:i32 b:i32) (:> i32)
  (match (a b)
    ((a 0) a)
    ((a b) (gcd b (mod a b)))))
```

### 3.6 Comparison: All Three Styles

**Style 1 - Explicit Match:**

```clojure
(deffn classify (n:i32) (:> string)
  (match n
    (n (when (< n 0)) "negative")
    (0 "zero")
    (n (when (> n 0)) "positive")))
```

**Style 2 - Function Heads:**

```clojure
(deffn classify (n:i32) (:> string)
  (n (when (< n 0)) "negative")
  (0 "zero")
  (n (when (> n 0)) "positive"))
```

**Style 3 - Regular Body (for comparison):**

```clojure
(deffn classify (n:i32) (:> string)
  (if (< n 0)
    "negative"
    (if (== n 0)
      "zero"
      "positive")))
```

All three are valid! Use whichever is clearest for your use case.

---

## 4. Examples

### 4.1 Fibonacci Sequence

Classic recursive example showing both styles:

**Explicit Match:**

```clojure
(deffn fibonacci (n:i32) (:> i32)
  "Calculate nth Fibonacci number"
  (match n
    (0 0)
    (1 1)
    (n (+ (fibonacci (- n 1)) (fibonacci (- n 2))))))
```

**Function Head Matching:**

```clojure
(deffn fibonacci (n:i32) (:> i32)
  "Calculate nth Fibonacci number"
  (0 0)
  (1 1)
  (n (+ (fibonacci (- n 1)) (fibonacci (- n 2)))))
```

Both are identical except for `(match n ...)`!

### 4.2 List Operations

**Length:**

```clojure
(deffn length (t) (items:(vec t)) (:> usize)
  "Calculate length of vector"
  ((vec::new) 0)
  ((cons _ rest) (+ 1 (length rest))))
```

**Map:**

```clojure
(deffn map (a b) (f:(fn (a) (:> b)) items:(vec a)) (:> (vec b))
  "Map function over vector"
  (_ (vec::new) (vec::new))
  (f (cons x xs) (cons (f x) (map f xs))))
```

**Filter:**

```clojure
(deffn filter (t) (pred:(fn (&t) (:> bool)) items:(vec t)) (:> (vec t))
  "Filter vector by predicate"
  (_ (vec::new) (vec::new))
  (pred (cons x xs) (when (pred &x)) (cons x (filter pred xs)))
  (pred (cons _ xs) (filter pred xs)))
```

### 4.3 Option Handling

**Unwrap with default:**

```clojure
(deffn unwrap-or (t) (opt:(option t) default:t) (:> t)
  "Unwrap option or return default"
  ((some value) _ value)
  ((none) default default))
```

**Map over option:**

```clojure
(deffn option-map (a b) (opt:(option a) f:(fn (a) (:> b))) (:> (option b))
  "Map function over option"
  ((none) _ (none))
  ((some value) f (some (f value))))
```

### 4.4 Result Handling

**Map result:**

```clojure
(deffn result-map (t e u)
  "Map function over successful result"

  ((err e:e) _) (:> (result) u e)
  (err e)

  ((ok value:t) f:(fn (t) (:> u))) (:> (result) u e)
  (ok (f value)))
```

**Chain results:**

```clojure
(deffn result-and-then (t e u)
  "Chain result-returning operations"

  ((err e:e) _) (:> (result) u e)
  (err e)

  ((ok value:t) f:(fn (t) (:> (result) u e))) (:> (result) u e)
  (f value))
```

### 4.5 Tree Operations

```clojure
(defenum tree (t)
  "Binary tree"
  (empty)
  (leaf t)
  (node t (tree t) (tree t)))

(deffn tree-size (t)
  "Count nodes in tree"

  ((tree::empty)) (:> usize)
  0

  ((tree::leaf _)) (:> usize)
  1

  ((tree::node _ left right)) (:> usize)
  (+ 1 (tree-size left) (tree-size right)))

(deffn tree-depth (t)
  "Calculate depth of tree"

  ((tree::empty)) (:> usize)
  0

  ((tree::leaf _)) (:> usize)
  1

  ((tree::node _ left right)) (:> usize)
  (+ 1 (max (tree-depth left) (tree-depth right))))

(deffn tree-map (a b)
  "Map function over tree values"

  ((tree::empty) _) (:> (tree) b)
  (tree::empty)

  ((tree::leaf value:a) f:(fn (a) (:> b))) (:> (tree) b)
  (tree::leaf (f value))

  ((tree::node value:a left right) f:(fn (a) (:> b))) (:> (tree) b)
  (tree::node (f value) (tree-map left f) (tree-map right f)))
```

### 4.6 Guards and Complex Conditions

**Number classification:**

```clojure
(deffn number-type
  "Classify a number"

  ((n:i32)) (:> string)
  (when (< n 0))
  "negative"

  ((0)) (:> string)
  "zero"

  ((n:i32)) (:> string)
  (when (== (mod n 2) 0))
  "even"

  ((n:i32)) (:> string)
  "odd")
```

**Bounded validation:**

```clojure
(deffn validate-age
  "Validate age is in reasonable range"

  ((age:u32)) (:> (result) u32 string)
  (when (== age 0))
  (err "Age cannot be zero")

  ((age:u32)) (:> (result) u32 string)
  (when (> age 150))
  (err "Age too high")

  ((age:u32)) (:> (result) u32 string)
  (ok age))
```

### 4.7 Multiple Parameters

**Binary operations:**

```clojure
(deffn gcd
  "Greatest common divisor"

  ((a:i32 0)) (:> i32)
  a

  ((a:i32 b:i32)) (:> i32)
  (gcd b (mod a b)))

(deffn min
  "Minimum of two values"

  ((x:i32 y:i32)) (:> i32)
  (when (<= x y))
  x

  ((x:i32 y:i32)) (:> i32)
  y)
```

**String operations:**

```clojure
(deffn string-append
  "Append strings with special cases"

  (("" s:string)) (:> string)
  s

  ((s:string "")) (:> string)
  s

  ((s1:string s2:string)) (:> string)
  (string::concat s1 s2))
```

### 4.8 Combining with Type Classes

```clojure
(deffn compare-and-describe (t)
  "Compare values and describe relationship"
  (where (t: (ord display)))

  ((a:&t b:&t)) (:> string)
  (when (< a b))
  (format "{} is less than {}" (a:display) (b:display))

  ((a:&t b:&t)) (:> string)
  (when (> a b))
  (format "{} is greater than {}" (a:display) (b:display))

  ((a:&t b:&t)) (:> string)
  (format "{} equals {}" (a:display) (b:display)))
```

### 4.9 Partial Application Style

```clojure
(deffn apply-or-default (t u)
  "Apply function if present, or return default"

  ((none) _ default:u) (:> u)
  default

  ((some f:(fn (t) (:> u))) value:t _) (:> u)
  (f value))
```

### 4.10 Merge Sort Example

Complete working example:

```clojure
(deffn merge (t)
  "Merge two sorted vectors"
  (where (t: ord))

  ;; Both empty
  ((vec::new) (vec::new)) (:> (vec) t)
  (vec::new)

  ;; Left empty
  ((vec::new) right:(vec t)) (:> (vec) t)
  right

  ;; Right empty
  ((left:(vec t) (vec::new))) (:> (vec) t)
  left

  ;; Both non-empty
  ((left:(vec t) right:(vec t))) (:> (vec) t)
  (let (x (vec:first &left)
        y (vec:first &right))
    (if (<= x y)
      (cons x (merge (vec:rest left) right))
      (cons y (merge left (vec:rest right))))))

(deffn merge-sort (t)
  "Sort a vector using merge sort"
  (where (t: ord))

  ;; Empty or single element
  ((v:(vec t))) (:> (vec) t)
  (when (<= (vec:len &v) 1))
  v

  ;; Multiple elements
  ((v:(vec t))) (:> (vec) t)
  (let (mid (/ (vec:len &v) 2)
        left (vec:take &v mid)
        right (vec:drop &v mid))
    (merge (merge-sort left) (merge-sort right))))
```

---

## 5. Macro Expansion

### 5.1 How It Works

The `deffn` macro detects multi-clause syntax and transforms it to a standard function with a `match` expression.

**Input:**

```clojure
(deffn fibonacci
  ((0)) (:> i32)
  0

  ((1)) (:> i32)
  1

  ((n:i32)) (:> i32)
  (+ (fibonacci (- n 1)) (fibonacci (- n 2))))
```

**Expansion:**

```clojure
(deffn fibonacci (n:i32) (:> i32)
  (match n
    (0 0)
    (1 1)
    (n (+ (fibonacci (- n 1)) (fibonacci (- n 2))))))
```

### 5.2 With Guards

**Input:**

```clojure
(deffn classify
  ((n:i32)) (:> string)
  (when (< n 0))
  "negative"

  ((0)) (:> string)
  "zero"

  ((n:i32)) (:> string)
  (when (> n 0))
  "positive")
```

**Expansion:**

```clojure
(deffn classify (n:i32) (:> string)
  (match n
    (n (when (< n 0)) "negative")
    (0 "zero")
    (n (when (> n 0)) "positive")))
```

### 5.3 Multiple Parameters

**Input:**

```clojure
(deffn gcd
  ((a:i32 0)) (:> i32)
  a

  ((a:i32 b:i32)) (:> i32)
  (gcd b (mod a b)))
```

**Expansion:**

```clojure
(deffn gcd (a:i32 b:i32) (:> i32)
  (match (a b)
    ((a 0) a)
    ((a b) (gcd b (mod a b)))))
```

### 5.4 Macro Implementation Sketch

```clojure
(defmacro deffn (name & body)
  "Define function with optional multi-clause pattern matching"

  ;; Check if this is multi-clause or single-clause
  (if (is-multi-clause? body)
    (expand-multi-clause name body)
    (expand-single-clause name body)))

(deffn is-multi-clause? (body)
  "Detect if body contains multiple (pattern) (:> type) clauses"
  (and (> (count body) 1)
       (every? #(starts-with? % '()
               (filter vector? body))))

(deffn expand-multi-clause (name clauses)
  "Expand multi-clause function to match expression"
  (let (;; Extract common elements
        ((type-params doc)) (extract-metadata clauses)
        param-names (extract-param-names clauses)
        return-type (extract-return-type clauses)

        ;; Build match clauses from function clauses
        match-clauses (map build-match-clause clauses))

    `(deffn ~name ~@type-params ~param-names -> ~return-type
       ~@(when doc (doc))
       (match ~(if (= (count param-names) 1)
                 (first param-names)
                 param-names)
         ~@match-clauses))))

(deffn build-match-clause (clause)
  "Transform function clause to match clause"
  (let ((pattern _ return-type & rest) clause
        ((guard body)) (if (and (list? (first rest))
                             (= (first (first rest)) 'when))
                       ((first rest) (rest rest))
                       ((nil rest))))
    `(~pattern ~@(when guard (guard)) ~@body)))
```

---

## 6. Design Rationale

### 6.1 Why Allow Both Styles?

Different situations call for different approaches:

**Use multi-clause when:**

- Function has clear, distinct cases
- Patterns are simple and visual separation helps
- Coming from ML/Erlang/Haskell background
- Base cases and recursive cases are distinct

**Use explicit match when:**

- Matching on expressions, not just parameters
- Deeply nested patterns
- Match is in the middle of function logic
- Prefer seeing all branches together

### 6.2 Consistency with Rust

Rust doesn't have multi-clause functions, but Oxur is a Lisp with Rust semantics, not a syntax clone. We can add Lisp-appropriate features that:

- Compile to valid Rust semantics
- Don't introduce new runtime behavior
- Feel natural to Lisp/functional programmers

This is analogous to how Rust has pattern matching in `match` but not in function heads, while OCaml has both.

### 6.3 Learning Curve

For Rust programmers learning Oxur:

- Single-clause syntax is familiar
- Multi-clause is optional sugar
- Both desugar to same code

For Lisp/functional programmers learning Oxur:

- Multi-clause feels familiar
- Can discover single-clause + match later
- Both work identically

### 6.4 Tooling Implications

- **Formatter**: Can convert between styles
- **LSP**: Show expansion on hover
- **Debugger**: Steps through expanded match
- **Documentation**: Can show either form

---

## 7. Comparison with Other Languages

### 7.1 Erlang/LFE

**LFE:**

```lfe
(defun fibonacci
  (0) 0
  (1) 1
  (n) (+ (fibonacci (- n 1)) (fibonacci (- n 2))))
```

**Oxur:**

```clojure
(deffn fibonacci
  ((0)) (:> i32)
  0

  ((1)) (:> i32)
  1

  ((n:i32)) (:> i32)
  (+ (fibonacci (- n 1)) (fibonacci (- n 2))))
```

**Differences:**

- Oxur requires explicit return types (Rust semantics)
- Oxur patterns in vectors `(...)` not just parens
- Oxur body on separate line (clearer with types)

### 7.2 Haskell

**Haskell:**

```haskell
fibonacci :: Int -> Int
fibonacci 0 = 0
fibonacci 1 = 1
fibonacci n = fibonacci (n - 1) + fibonacci (n - 2)
```

**Oxur:**

```clojure
(deffn fibonacci
  ((0)) (:> i32)
  0

  ((1)) (:> i32)
  1

  ((n:i32)) (:> i32)
  (+ (fibonacci (- n 1)) (fibonacci (- n 2))))
```

**Differences:**

- Oxur uses s-expressions
- Oxur inline return type per clause
- Oxur patterns in vectors

### 7.3 OCaml/SML

**OCaml:**

```ocaml
let rec fibonacci = function
  | 0 (:> 0)
  | 1 (:> 1)
  | n (:> fibonacci) (n - 1) + fibonacci (n - 2)
```

**Oxur:**

```clojure
(deffn fibonacci
  ((0)) (:> i32)
  0

  ((1)) (:> i32)
  1

  ((n:i32)) (:> i32)
  (+ (fibonacci (- n 1)) (fibonacci (- n 2))))
```

**Similarities:**

- Pattern matching syntax
- Guards supported
- Recursive definitions

**Differences:**

- Oxur is s-expressions
- Oxur has explicit types
- Oxur uses vectors for patterns

### 7.4 Elixir

**Elixir:**

```elixir
def fibonacci(0), do: 0
def fibonacci(1), do: 1
def fibonacci(n), do: fibonacci(n - 1) + fibonacci(n - 2)
```

**Oxur:**

```clojure
(deffn fibonacci
  ((0)) (:> i32)
  0

  ((1)) (:> i32)
  1

  ((n:i32)) (:> i32)
  (+ (fibonacci (- n 1)) (fibonacci (- n 2))))
```

**Similarities:**

- Multiple clause definitions
- Pattern matching
- Recursion

---

## 8. Implementation Notes

### 8.1 Parser Changes

The parser needs to recognize multi-clause patterns:

```
function-def := (deffn NAME type-params? clause+)

clause := pattern-clause | single-clause

pattern-clause := (pattern+) (:> type) guard? body+

single-clause := (param+) (:> type) docstring? body+

guard := (when expr)
```

### 8.2 Type Inference

Each clause must:

1. Have compatible parameter patterns
2. Match the function's type signature
3. Have return type compatible with other clauses

The macro validates:

- All clauses have same number of parameters
- All return types match
- Patterns are exhaustive (warning if not)

### 8.3 Error Messages

Good error messages are crucial:

```
Error: Clause parameter count mismatch
  --> fibonacci.oxur:5:3
   |
3  | (0) (:> i32)
   | --- First clause has 1 parameter
5  | (x:i32 y:i32) (:> i32)
   | ^^^^^^^^^^^^^ This clause has 2 parameters
   |
   = help: All clauses must have the same number of parameters
```

### 8.4 Exhaustiveness Checking

The compiler should warn about non-exhaustive patterns:

```
Warning: Non-exhaustive patterns
  --> classify.oxur:1:1
   |
1  | (deffn classify
   | ^^^^^^^^^^^^^^^ Missing patterns for negative numbers
   |
   = help: Add clause for `(n:i32) (when (< n 0)) ...`
```

---

## 9. Future Considerations

### 9.1 Optimization

Multi-clause functions could enable optimizations:

- **Decision tree compilation**: Smart ordering of checks
- **Jump tables**: For literal patterns
- **Inlining**: Each clause separately

### 9.2 Documentation Generation

Tools could generate nice documentation:

```markdown
## fibonacci

```clojure
(deffn fibonacci
  (0) (:> i32)
  (1) (:> i32)
  (n:i32) (:> i32))
```

Calculate nth Fibonacci number.

**Cases:**

- `0` → Returns 0 (base case)
- `1` → Returns 1 (base case)
- `n` → Recursive case

```

### 9.3 REPL Support

The REPL could show clauses:

```

oxur> :clauses fibonacci
fibonacci has 3 clauses:

  1. (0) (:> i32)
  2. (1) (:> i32)
  3. (n:i32) (:> i32)

oxur> :expand fibonacci
Expands to:
(deffn fibonacci (n:i32) (:> i32)
  (match n
    (0 0)
    (1 1)
    (n (+ (fibonacci (- n 1)) (fibonacci (- n 2))))))

```

### 9.4 Partial Application

Could support curried style:

```clojure
(deffn add
  ((0)) (:> (fn) (i32) (:> i32))
  identity

  ((x:i32)) (:> (fn) (i32) (:> i32))
  (fn (y) (+ x y)))
```

### 9.5 Type-Directed Dispatch

Could use types in patterns:

```clojure
(deffn process
  ((x:string)) (:> result)
  (process-string x)

  ((x:i32)) (:> result)
  (process-int x))
```

Though this might be better served by traits.

---

## 10. Complete Syntax Summary

### The Three Forms

**1. Regular Function (single expression or sequence):**

```clojure
(deffn add (x:i32 y:i32) (:> i32)
  (+ x y))

(deffn complex-calc (x:f64) (:> f64)
  (let ((squared (* x x))
        (doubled (* squared 2)))
    (sqrt doubled)))
```

**2. Explicit Match:**

```clojure
(deffn factorial (n:i32) (:> i32)
  (match n
    (0 1)
    (n (* n (factorial (- n 1))))))
```

**3. Function Head Matching (sugar for #2):**

```clojure
(deffn factorial (n:i32) (:> i32)
  (0 1)
  (n (* n (factorial (- n 1)))))
```

### Conversion Between Styles

**To convert from explicit match to function heads:**
Remove `(match param-name ...)` wrapper, keep clauses.

**To convert from function heads to explicit match:**
Wrap clauses in `(match param-name ...)`.

### Macro Detection Algorithm

```clojure
(defmacro deffn (name params return-type & body)
  (cond
    ;; 1. Explicit match - first form is match
    ((and (list? (first body))
          (= 'match (first (first body))))
     (expand-explicit-match name params return-type body))

    ;; 2. Function heads - body looks like clauses
    ((all-pattern-clauses? body)
     (expand-function-heads name params return-type body))

    ;; 3. Regular function body
    (else
     (expand-regular-function name params return-type body))))
```

### Pattern Clause Detection

```clojure
(deffn all-pattern-clauses? (body)
  "Check if body consists of pattern clauses"
  (and (not (empty? body))
       (every? could-be-clause? body)))

(deffn could-be-clause? (form)
  "Heuristic: does this look like a pattern clause?"
  (and (list? form)
       (>= (length form) 2)
       (could-be-pattern? (first form))))

(deffn could-be-pattern? (form)
  "Could this be a pattern?"
  (or (literal? form)          ; 0, 1, "hello"
      (symbol? form)            ; n, x, _
      (and (list? form)
           (or (= (first form) 'when)   ; guard
               (symbol? (first form)))))) ; (some x), (cons a b)
```

### Complete Example Comparison

**Task**: Implement a simple calculator

**Style 1 - Regular Function:**

```clojure
(deffn calc (op:&str x:i32 y:i32) (:> (result i32 string))
  (if (== op "+")
    (ok (+ x y))
    (if (== op "-")
      (ok (- x y))
      (if (== op "*")
        (ok (* x y))
        (if (and (== op "/") (!= y 0))
          (ok (/ x y))
          (err "Invalid operation or division by zero"))))))
```

**Style 2 - Explicit Match:**

```clojure
(deffn calc (op:&str x:i32 y:i32) (:> (result i32 string))
  (match op
    ("+" (ok (+ x y)))
    ("-" (ok (- x y)))
    ("*" (ok (* x y)))
    ("/" (when (!= y 0)) (ok (/ x y)))
    (_ (err "Invalid operation"))))
```

**Style 3 - Function Heads:**

```clojure
(deffn calc (op:&str x:i32 y:i32) (:> (result i32 string))
  ("+" (ok (+ x y)))
  ("-" (ok (- x y)))
  ("*" (ok (* x y)))
  ("/" (when (!= y 0)) (ok (/ x y)))
  (_ (err "Invalid operation")))
```

All three are valid! Style 2 and 3 are nearly identical.

---

## 11. Conclusion

## 11. Conclusion

The refined function head matching design achieves **perfect alignment** between explicit `match` and pattern-based function definitions:

✅ **Minimal Syntax Difference** - Only `(match param ...)` wrapper distinguishes the styles
✅ **Zero Repetition** - Return type declared once in function signature
✅ **Easy Conversion** - Add/remove match wrapper to switch styles
✅ **Moderate Complexity** - Macro uses simple heuristics to detect style
✅ **Maximum Flexibility** - Three valid styles for different use cases
✅ **Excellent DX** - Clean, readable, consistent

### Key Innovation

By recognizing that Rust requires all match arms to return the same type, we eliminated the need to repeat `(:> type)` in every clause. This makes function head matching **visually identical** to explicit match, except for the wrapper.

### The Result

```clojure
;; These are nearly identical:
(deffn fib (n:i32) (:> i32)
  (match n (0 0) (1 1) (n (+ (fib (- n 1)) (fib (- n 2))))))

(deffn fib (n:i32) (:> i32)
  (0 0) (1 1) (n (+ (fib (- n 1)) (fib (- n 2)))))
```

This strengthens Oxur's position as a **Lisp for Rust** by bringing functional programming idioms in the cleanest possible form, while maintaining Rust's type safety and performance guarantees.

**Function head matching: Maximum elegance, minimal magic.** ✨

---

## Appendix: Side-by-Side Comparison

### Factorial

**Explicit Match:**

```clojure
(deffn factorial (n:i32) (:> i32)
  (match n
    (0 1)
    (n (* n (factorial (- n 1))))))
```

**Function Heads:**

```clojure
(deffn factorial (n:i32) (:> i32)
  (0 1)
  (n (* n (factorial (- n 1)))))
```

### List Sum

**Explicit Match:**

```clojure
(deffn sum (items:(vec i32)) (:> i32)
  (match items
    ((vec::new) 0)
    ((cons x rest) (+ x (sum rest)))))
```

**Function Heads:**

```clojure
(deffn sum (items:(vec i32)) (:> i32)
  ((vec::new) 0)
  ((cons x rest) (+ x (sum rest))))
```

### Option Unwrap

**Explicit Match:**

```clojure
(deffn unwrap-or (t) (opt:(option t) default:t) (:> t)
  (match opt
    ((some value) value)
    ((none) default)))
```

**Function Heads:**

```clojure
(deffn unwrap-or (t) (opt:(option t) default:t) (:> t)
  ((some value) value)
  ((none) default))
```

---

*Addendum Version: 1.0*
*Date: January 1, 2025*
*Status: Design Proposal*

**Next Steps:**

1. Implement `deffn` macro with multi-clause support
2. Add exhaustiveness checking
3. Create formatter rules for both styles
4. Update LSP to show expansions
5. Add examples to standard library

**Happy pattern matching!** 🎉🎨✨
