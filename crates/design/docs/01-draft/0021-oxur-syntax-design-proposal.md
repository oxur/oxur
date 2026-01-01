---
number: 21
title: "Oxur Syntax Design Proposal"
author: "choosing inline"
created: 2025-12-31
updated: 2025-12-31
state: Draft
supersedes: null
superseded-by: null
version: 1.0
---

# Oxur Syntax Design Proposal

**Authors**: Design discussion between Duncan McGreggor  and Claude.ai (Sonnet 4.5)
**Date**: December 31, 2025 / January 1, 2026
**Status**: Proposal

---

## Table of Contents

1. [Introduction](#introduction)
2. [The Sample Project: A Greeting System](#the-sample-project)
3. [Language Comparisons](#language-comparisons)
4. [Design Aesthetics and Philosophy](#design-aesthetics)
5. [The Oxur Syntax](#the-oxur-syntax)
6. [Comprehensive Examples](#comprehensive-examples)
7. [Future Directions](#future-directions)
8. [Appendix: Full Source Listings](#appendix)

---

## 1. Introduction

### What is Oxur?

Oxur is a Lisp-syntax language that compiles to Rust, bringing together:

- The **elegance and expressiveness** of Lisp (s-expressions, macros, REPL)
- The **safety and performance** of Rust (ownership, borrowing, zero-cost abstractions)
- A **pragmatic syntax** that makes types visible where they matter

### Why Oxur?

Rust's type system and ownership model are beautiful and powerful, but the syntax can be verbose. Lisp's homoiconicity and macro system are unparalleled for metaprogramming, but most Lisps lack modern type systems and memory safety guarantees.

Oxur asks: **What if we could have both?**

### Design Goals

1. **Safety First**: Full Rust semantics - ownership, borrowing, lifetimes
2. **Lisp Heritage**: S-expressions, macros, REPL-driven development
3. **Inline Types**: Types annotated where things happen (not separate declarations)
4. **Zero Compromise**: Compile to efficient Rust, no runtime overhead
5. **Pragmatic Beauty**: Clean syntax that reads well and writes naturally

---

## 2. The Sample Project: A Greeting System

To explore syntax possibilities, we created a simple greeting system in Rust and then translated it to various Lisp dialects. This project demonstrates:

- Basic functions with type signatures
- Borrowing vs. ownership
- Optional types
- Pattern matching
- Command-line argument parsing
- String manipulation

### 2.1 Rust Implementation

The original Rust code provides our baseline for semantics and features.

**Project Structure:**

```
oxur_syntax_exploration/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   └── main.rs
└── README.md
```

**Key Rust Concepts Demonstrated:**

- Borrowing: `fn format_greeting(name: &str) -> String`
- Ownership: `fn create_personalized_greeting(name: String, ...) -> String`
- Option types: `enthusiasm_level: Option<u32>`
- Pattern matching: `match enthusiasm_level { Some(level) => ..., None => ... }`
- Method chaining: `greeting.chars().count()`

See [Appendix A](#appendix-a-rust-source) for full source code.

---

## 3. Language Comparisons

We explored how different Lisp dialects handle typing and Rust-like concepts.

### 3.1 Racket (Dynamic)

**Key Features:**

- Dynamically typed by default
- Immutable strings
- Pattern matching via `match` library
- No ownership/borrowing - garbage collected

**Example:**

```racket
(define (format-greeting name)
  (let ((title (determine-title name)))
    (format "Hello, ~a ~a!" title name)))
```

**Insights:**

- Clean and simple
- Types completely absent
- Good for comparison but not our target

### 3.2 Typed Racket (Static)

**Key Features:**

- Static typing with inference
- Type annotations: `(: function-name (-> ArgType ReturnType))`
- Option types: `(Option Integer)`
- Pattern matching on sum types

**Example:**

```racket
(: format-greeting (-> String String))
(define (format-greeting name)
  (let ((title (determine-title name)))
    (format "Hello, ~a ~a!" title name)))
```

**Insights:**

- Types separate from definition (not ideal)
- Good type system but verbose
- Inspired our `: Type` notation

### 3.3 Shen (Gradual)

**Key Features:**

- Sequent calculus type system
- Types declared separately: `{string --> string}`
- Pattern matching in function definitions
- Optional type checking

**Example:**

```shen
(define format-greeting
  {string --> string}
  Name -> (let Title (determine-title Name)
            (make-string "Hello, ~A ~A!" Title Name)))
```

**Insights:**

- Elegant separation of types from implementation
- Multiple function clauses for pattern matching
- Too mathematical for our pragmatic goals

### 3.4 Coalton (Hindley-Milner in Common Lisp)

**Key Features:**

- ML-style type inference
- Types: `(declare name (Type -> Type))`
- Pattern matching via `match`
- Embedded DSL in Common Lisp

**Example:**

```lisp
(declare format-greeting (String -> String))
(define (format-greeting name)
  (let ((title (determine-title name)))
    (str:concat "Hello, " title " " name "!")))
```

**Insights:**

- Powerful type system
- `declare` syntax not ideal (types separate from code)
- Showed that strong typing in Lisp is possible

### 3.5 LFE (Lisp Flavored Erlang)

**Key Features:**

- Dialyzer type specs (success typing)
- Type specs: `(defspec function arity ((input-types) (return-type)))`
- Pattern matching in function heads
- Actor model concurrency

**Example:**

```lfe
(defspec format-greeting 1
  (((list)) (list)))

(defun format-greeting (name)
  (let ((title (determine-title name)))
    (++ "Hello, " title " " name "!")))
```

**Insights:**

- Multi-clause function definitions are elegant
- Type specs optional (gradual typing)
- Inspired our `match` syntax with double-parens

### 3.6 Clojure with core.typed

**Key Features:**

- TWO type systems: core.typed (safety) + hints (performance)
- Type annotations: `(ann name (Input -> Output))`
- Type hints: `^Type` metadata
- Gradual typing as library

**Example:**

```clojure
(t/ann format-greeting [String -> String])
(defn format-greeting
  [name]
  (let [title (determine-title name)]
    (str "Hello, " title " " name "!")))

;; With type hints for performance
(defn format-greeting-fast
  ^String [^String name]
  ...)
```

**Insights:**

- **BREAKTHROUGH**: The `^Type` hint syntax!
- Inline metadata feels natural in Lisp
- Types where things happen, not separate
- This became our 40% influence

See [Appendix B-G](#appendix-b-g-lisp-translations) for full translations.

---

## 4. Design Aesthetics and Philosophy

### 4.1 The 40/10/50 Formula

After exploring all options, we settled on a blend:

- **40% Clojure type hints** - `^Type` metadata syntax (inline, natural)
- **10% Typed Racket** - `: Type` notation (clean, readable)
- **50% Rust AST** - Ownership semantics, but less verbose

### 4.2 Why Inline Types?

**The Problem with Separate Type Declarations:**

Many typed Lisps separate type information from definitions:

```lisp
;; Coalton style - types separate
(declare format-greeting (String -> String))
(define (format-greeting name) ...)

;; Typed Racket style - types separate
(: format-greeting (-> String String))
(define (format-greeting name) ...)

;; Clojure core.typed - types separate
(ann format-greeting [String -> String])
(defn format-greeting [name] ...)
```

**Problems:**

- Have to look in two places to understand a function
- Types can drift from implementation
- Extra ceremony and boilerplate
- Not how Go or Rust do it

**What We Love About Rust and Go:**

```rust
// Rust - types where things happen
fn format_greeting(name: &str) -> String { ... }
```

```go
// Go - types where things happen
func formatGreeting(name string) string { ... }
```

Types are **inline**, **visible**, and **immediate**.

### 4.3 The Clojure Hint Insight

Clojure's type hints showed us the way:

```clojure
(defn my-func ^String [^String name]
  ...)
```

The `^Type` metadata is:

- **Inline** - right where the binding happens
- **Non-intrusive** - doesn't clutter the structure
- **Lispy** - metadata is a Lisp tradition

But we can improve on it!

### 4.4 Our Syntax Innovation

Combining the best ideas:

```clojure
(deffn format-greeting
  (name:&str) -> string
  "Docstring here"
  ...)
```

**Key decisions:**

1. **`name:type`** - Inspired by Rust's `: Type` but compact
2. **`-> returntype`** - Clear return type separator
3. **`::` vs `:`** - Module paths vs. types/methods
4. **Lowercase types** - True Lisp fashion
5. **`deffn`** - Consistent with `defstruct`, `defenum`, etc.

### 4.5 Why `deffn` Not `defun` or `defn`?

Traditional Lisp has inconsistent naming:

- `defun` - "define function" (elided second 'f')
- `defmacro` - "define macro" (both words present)
- `defstruct` - "define struct" (both words present)

**Why this inconsistency?** Historical accident. `defun` comes from very early Lisp.

**Oxur chooses logic over tradition:**

- `deffn` - "define function" (consistent!)
- `defmacro` - "define macro"
- `defstruct` - "define struct"
- `defenum` - "define enum"
- `deftrait` - "define trait"
- `defimpl` - "define implementation"

**Benefits:**

- Instant recognition ("This is Oxur, not Clojure/Scheme")
- Logical consistency
- No special cases to remember

### 4.6 Module Paths: `::` vs `:`

Following Rust's semantics:

**`::`** for static/module paths (compile-time):

```clojure
(use std::collections::hashmap)
(hashmap::new)              ; HashMap::new()
(string::from "hello")      ; String::from()
(option::some 42)           ; Option::Some(42)
```

**`:`** for types and instance methods:

```clojure
(name:string)               ; Type annotation
(string:len s)              ; s.len() - instance method
(vec:push! v item)          ; v.push(item)
```

**Why this works:**

- Context disambiguates (call position vs. binding position)
- Matches Rust's `::` for associated functions
- Feels natural to Lisp programmers (`:` for keywords/qualification)

---

## 5. The Oxur Syntax

### 5.1 Core Example

The function that started it all, now in Oxur:

```clojure
(use std::option::{option some none})

(deffn greet-user
  (name:string count:(option u32)) -> (result string error)
  "Greets a user with optional repetition"
  (let (base (string::from "Hello, ")           ; static String::from
        greeting (string:push-str base name)     ; instance method
        final (match count
                ((some n) (string:repeat greeting n))
                ((none) greeting))
    (ok final)))
```

**Breaking it down:**

1. **Import**: `(use std::option::{option some none})` - Rust-style paths
2. **Function definition**: `(deffn greet-user ...)` - Consistent naming
3. **Parameters**: `(name:string count:(option u32))` - Inline types
4. **Return**: `-> (result string error)` - Clear separator
5. **Docstring**: Right after signature, Lisp tradition
6. **Static call**: `(string::from "Hello, ")` - Associated function
7. **Instance method**: `(string:push-str base name)` - Method call
8. **Pattern matching**: `(match count ((some n) ...) ((none) ...))` - LFE-inspired
9. **Constructor**: `(ok final)` - Enum variant

### 5.2 Type Annotations

**Function parameters:**

```clojure
(name:string)              ; Owned value
(name:&str)                ; Borrowed reference
(name:&mut string)         ; Mutable borrow
```

**Return types:**

```clojure
-> string                  ; Owned return
-> &str                    ; Borrowed return
-> ()                      ; Unit type
-> (option i32)            ; Generic type
```

**Local bindings:**

```clojure
(let (x:i32 42)            ; Explicit type
      (y (+ x 1))          ; Inferred type
  ...)
```

### 5.3 Ownership and Borrowing

**Taking ownership:**

```clojure
(deffn take-string (s:string) -> usize
  (string:len &s))  ; Borrow s to call len
  ;; s dropped here
```

**Borrowing immutably:**

```clojure
(deffn borrow-string (s:&str) -> usize
  (string:len s))   ; s is already borrowed
  ;; s not dropped, caller still owns it
```

**Borrowing mutably:**

```clojure
(deffn modify-string (s:&mut string) -> ()
  (string:push-str s " world!")
  ())
```

**Explicit borrowing in calls:**

```clojure
(let (s (string::from "hello"))
  (borrow-string &s)       ; Borrow s
  (println! "{}" s))       ; Still usable
```

**Explicit cloning:**

```clojure
(let (s (string::from "hello")
      copy (clone s))      ; Explicit clone
  (take-string s)          ; s moved
  (println! "{}" copy))    ; copy still usable
```

### 5.4 Structs

**Definition:**

```clojure
(defstruct person
  "A person with name and age"
  (name:string
   age:u32
   email:(option string)))
```

**Associated functions (static):**

```clojure
(defimpl person
  (deffn new (name:string age:u32) -> person
    "Create a new person"
    (person name age (none))))
```

**Instance methods:**

```clojure
(defimpl person
  (deffn greet (self:&person) -> string
    "Instance method - borrows self"
    (format "Hi, I'm {}" (. self name)))

  (deffn have-birthday (self:&mut person) -> ()
    "Mutable instance method"
    (set! (. self age) (+ (. self age) 1))))
```

**Usage:**

```clojure
(let (mut p (person::new "Alice" 30))
  (println! "{}" (p:greet))          ; Instance method
  (p:have-birthday)                  ; Mutable method
  (println! "{}" (. p age)))         ; Field access: 31
```

### 5.5 Enums

**Simple enum:**

```clojure
(defenum shape
  "Different geometric shapes"
  (circle f64)                 ; Tuple variant
  (rectangle f64 f64)
  (point))                     ; Unit variant
```

**Enum with named fields:**

```clojure
(defenum message
  (quit)
  (move {x:i32 y:i32})
  (write string)
  (change-color {r:u8 g:u8 b:u8}))
```

**Pattern matching:**

```clojure
(deffn process-message (msg:message) -> ()
  (match msg
    ((quit)
     (println! "Quitting..."))

    ((move {x y})
     (println! "Moving to ({}, {})" x y))

    ((write text)
     (println! "Text: {}" text))

    ((change-color {r g b})
     (println! "Color: RGB({}, {}, {})" r g b))))
```

### 5.6 Pattern Matching

**Basic matching:**

```clojure
(match value
  ((some x) (format "Got: {}" x))
  ((none) "Nothing"))
```

**With guards:**

```clojure
(match value
  ((some x) (when (> x 0)) "Positive")
  ((some x) (when (< x 0)) "Negative")
  ((some 0) "Zero")
  ((none) "None"))
```

**Nested patterns:**

```clojure
(match result
  ((ok (some value)) (println! "Success: {}" value))
  ((ok (none)) (println! "Success but empty"))
  ((err e) (println! "Error: {}" e)))
```

**Multiple patterns (or-patterns):**

```clojure
(match n
  ((| 0 1) "Small")
  ((2..=10) "Medium")
  (_ "Large"))
```

**Destructuring:**

```clojure
(let ((point x y) p)
  (+ x y))

(match shape
  ((circle r) (* 3.14159 r r))
  ((rectangle w h) (* w h)))
```

### 5.7 Generics

**Generic function:**

```clojure
(deffn identity (t) (x:t) -> t
  "Generic identity function"
  x)
```

**Multiple type parameters:**

```clojure
(deffn pair (a b) (first:a second:b) -> (a b)
  "Create a pair"
  (first second))
```

**Generic struct:**

```clojure
(defstruct point (t)
  (x:t y:t))

(defimpl (t) point
  (deffn new (x:t y:t) -> (point t)
    (point x y)))
```

**With trait bounds:**

```clojure
(deffn find-max (t) (items:&(vec t)) -> (option &t)
  (where (t: ord))
  "Find maximum element"
  (items:iter)
  (iter:max))
```

### 5.8 Traits

**Definition:**

```clojure
(deftrait summary
  "Objects that can be summarized"

  (deffn summarize (self:&self) -> string
    "Return a summary"))
```

**Implementation:**

```clojure
(defimpl summary person
  (deffn summarize (self:&person) -> string
    (format "{} (age {})" (. self name) (. self age))))
```

**Default methods:**

```clojure
(deftrait display
  (deffn display (self:&self) -> string
    "Display representation")

  (deffn display-with-prefix (self:&self prefix:&str) -> string
    "Default implementation"
    (format "{}{}" prefix (self:display))))
```

**Trait bounds in functions:**

```clojure
(deffn print-summary (item:&t) -> ()
  (where (t: summary))
  (println! "{}" (item:summarize)))

(deffn compare-and-display (a:&t b:&t) -> ()
  (where (t: (ord display)))
  ...)
```

### 5.9 Lifetimes

**Explicit lifetime annotations (rare):**

```clojure
(deffn longest ('a)
  (x:&'a str y:&'a str) -> &'a str
  "Return the longest string"
  (if (> (string:len x) (string:len y))
    x
    y))
```

**Struct with lifetime:**

```clojure
(defstruct excerpt ('a)
  (part:&'a str))

(defimpl ('a) excerpt
  (deffn get-part (self:&(excerpt 'a)) -> &'a str
    (. self part)))
```

**Multiple lifetimes:**

```clojure
(deffn compare ('a 'b)
  (x:&'a str y:&'b str) -> &'a str
  (where ('a: 'b))
  ...)
```

### 5.10 Error Handling

**Result type:**

```clojure
(defenum file-error
  (not-found string)
  (permission-denied)
  (io-error string))

(deffn read-file (path:&str) -> (result string file-error)
  (try
    (ok (fs::read-to-string path))
    (catch io-error e
      (err (file-error::io-error e)))))
```

**Error propagation with `?`:**

```clojure
(deffn read-and-process (path:&str) -> (result usize file-error)
  (let (contents (read-file path)?)     ; ? propagates errors
    (ok (string:len &contents))))
```

**Or using threading macro:**

```clojure
(deffn chain-operations (path:&str) -> (result () error)
  (-> (read-file path)?
      (process-contents)?
      (write-output)?
      (ok)))
```

### 5.11 Macros

**Simple macro:**

```clojure
(defmacro unless (condition body)
  "Execute body unless condition is true"
  `(if (not ~condition)
     ~body
     ()))
```

**Macro with destructuring:**

```clojure
(defmacro when-let (binding then else)
  "Execute then if binding succeeds"
  (let ((var expr) binding)
    `(match ~expr
       ((some ~var) ~then)
       ((none) ~else))))
```

**Variadic macro:**

```clojure
(defmacro vec! (& items)
  "Create a vector from items"
  `(vec::from (~@items)))
```

**Complex macro:**

```clojure
(defmacro defbuilder (name & fields)
  "Generate builder pattern"
  `(do
     (defstruct ~(builder-name name)
       ~@(builder-fields fields))

     (defimpl ~(builder-name name)
       ~@(builder-methods fields)

       (deffn build (self:~name-builder) -> (result ~name string)
         ~(build-logic fields)))))
```

### 5.12 Module System

**Module declaration:**

```clojure
(module myapp::greeting
  "Greeting functionality")
```

**Imports:**

```clojure
;; Import specific items
(use std::collections::hashmap)
(use std::io::{stdin stdout})
(use std::option::{option some none})

;; Import with alias
(use std::collections::hashmap :as map)

;; Import all from module
(use mylib::utils::*)

;; Re-export
(pub use std::option::option)
```

**Visibility:**

```clojure
;; Public function
(pub deffn greet (name:&str) -> string
  ...)

;; Private function (default)
(deffn helper (x:i32) -> i32
  ...)

;; Public struct, private fields
(pub defstruct person
  (name:string           ; private
   (pub age:u32)))       ; public

;; Pub in module
(pub mod utils
  (pub deffn helper () -> () ...))
```

**Exports:**

```clojure
(export
  greet
  person
  shape)
```

---

## 6. Comprehensive Examples

### 6.1 Complete Program Structure

```clojure
;;;; myapp.oxur - A complete Oxur application

(module myapp
  "Application entry point")

;; Imports
(use std::io)
(use std::collections::hashmap)
(use std::result::{result ok err})

;; Type definitions
(defstruct config
  (name:string
   port:u16
   debug:bool))

(defenum app-error
  (config-error string)
  (network-error string)
  (database-error string))

;; Trait definition
(deftrait runnable
  (deffn run (self:&self) -> (result () app-error)))

;; Implementation
(defimpl runnable config
  (deffn run (self:&config) -> (result () app-error)
    (println! "Starting {} on port {}" (. self name) (. self port))
    (ok ())))

;; Helper functions
(deffn parse-config (args:&(vec string)) -> (result config app-error)
  (if (< (vec:len args) 2)
    (err (app-error::config-error "Missing arguments"))
    (ok (config
          (clone (vec:get args 0))
          (parse-int (vec:get args 1))?
          true))))

;; Main entry point
(deffn main (args:(vec string)) -> (result () i32)
  (match (parse-config &args)
    ((ok cfg)
     (match (cfg:run)
       ((ok _) (ok ()))
       ((err e)
        (println! "Error: {:?}" e)
        (err 1))))
    ((err e)
     (println! "Config error: {:?}" e)
     (err 1))))
```

### 6.2 Working with Collections

```clojure
(deffn collection-examples () -> ()
  ;; Vectors
  (let (mut v (vec::new))
    (vec:push &mut v 1)
    (vec:push &mut v 2)
    (vec:push &mut v 3)

    (println! "Vector: {:?}" v)
    (println! "Length: {}" (vec:len &v))
    (println! "First: {:?}" (vec:first &v)))

  ;; HashMaps
  (let (mut scores (hashmap::new))
    (hashmap:insert &mut scores "Blue" 10)
    (hashmap:insert &mut scores "Red" 50)

    (match (hashmap:get &scores "Blue")
      ((some score) (println! "Blue: {}" score))
      ((none) (println! "No score")))

    (for ((key value) scores)
      (println! "{}: {}" key value)))

  ;; Iterator chains
  (let (result (-> (vec::from (1 2 3 4 5))
                   (iter)
                   (iter:filter (fn (x) (> x 2)))
                   (iter:map (fn (x) (* x 2)))
                   (iter:collect)))
    (println! "Filtered and doubled: {:?}" result)))
```

### 6.3 Async/Await (Future Feature)

```clojure
(deffn async fetch-url (url:&str) -> (result string http-error)
  "Async function to fetch URL"
  (let (response (http::get url).await?
        text (response:text).await?)
    (ok text)))

(deffn async process-urls (urls:(vec string)) -> ()
  "Process multiple URLs concurrently"
  (let (mut tasks (vec::new))

    ;; Spawn tasks
    (for (url urls)
      (vec:push &mut tasks (spawn (fetch-url &url))))

    ;; Await results
    (for (task tasks)
      (match (task.await)
        ((ok data) (println! "Got: {}" data))
        ((err e) (println! "Error: {:?}" e))))))
```

### 6.4 Concurrency Primitives

```clojure
;; Threads
(deffn thread-example () -> ()
  (let (handle (thread::spawn
                 (fn move ()
                   (println! "Hello from thread!")
                   42)))

    (println! "Hello from main!")

    (let (result (thread:join handle))
      (println! "Thread result: {}" result))))

;; Channels
(deffn channel-example () -> ()
  (let ((tx rx) (mpsc::channel))

    (thread::spawn
      (fn move ()
        (mpsc:send &tx "message")))

    (match (mpsc:recv &rx)
      ((ok msg) (println! "Received: {}" msg))
      ((err _) (println! "Channel closed")))))

;; Shared state
(deffn shared-state () -> ()
  (let (counter (arc::new (mutex::new 0))
        mut handles (vec::new))

    (for (_ (range 10))
      (let (counter-clone (arc::clone &counter))
        (vec:push &mut handles
          (thread::spawn
            (fn move ()
              (let (mut num (mutex:lock &counter-clone))
                (set! num (+ num 1))))))))

    (for (handle handles)
      (thread:join handle))

    (println! "Final count: {}" (mutex:lock &counter))))
```

---

## 7. Future Directions

### 7.1 OTP-Style Libraries

While Oxur stays close to Rust's low-level primitives, we envision building Erlang/OTP-inspired abstractions as pure Oxur libraries:

```clojure
(use oxur-otp::gen-server)
(use oxur-otp::supervisor)

(defgen-server my-server
  "OTP-style gen_server"

  (deffn init (args) -> (result state error)
    (ok (make-state args)))

  (deffn handle-call (request from state) -> (reply state)
    (match request
      ((get-value) (reply (. state value) state))
      ((set-value v) (reply :ok (assoc state :value v)))))

  (deffn handle-cast (msg state) -> (noreply state)
    (match msg
      ((increment) (noreply (update state :value inc)))))

  (deffn terminate (reason state) -> ()
    (println! "Shutting down: {:?}" reason)))

(defsupervisor my-app
  (strategy :one-for-one)
  (children
    ((my-server :permanent)
     (my-worker :temporary))))
```

**Key Point**: These are *just Oxur libraries* using Rust primitives underneath. No special runtime, just macros and abstractions.

### 7.2 REPL and Interactive Development

Oxur should support a full REPL experience:

```
oxur> (deffn add (x:i32 y:i32) -> i32 (+ x y))
#'user/add

oxur> (add 2 3)
5

oxur> (defstruct point (x:i32 y:i32))
user/point

oxur> (let (p (point 10 20)) (. p x))
10

oxur> (:type add)
(i32 i32) -> i32
```

### 7.3 Tooling

- **Formatter**: `oxurfmt` - opinionated code formatting
- **Linter**: `oxur-lint` - catch common mistakes
- **Language Server**: LSP support for editors
- **Package Manager**: Integration with Cargo
- **Documentation**: Doc generation from docstrings

### 7.4 Interop

**Calling Rust from Oxur**:

```clojure
(extern "rust"
  (deffn my_rust_function (x:i32) -> i32))

(my-rust-function 42)
```

**Calling Oxur from Rust**:

```rust
// Generated Rust code from Oxur
pub fn greet_user(name: String) -> String {
    // ...
}
```

### 7.5 Advanced Features

- **Procedural Macros**: Compile-time code generation
- **Attribute Macros**: Derive, conditional compilation
- **const Functions**: Compile-time evaluation
- **Inline Assembly**: When you need it (rare)
- **Custom Allocators**: Fine-grained memory control

---

## 8. Appendix: Full Source Listings

### Appendix A: Rust Source

#### Cargo.toml

```toml
[package]
name = "oxur_syntax_exploration"
version = "0.1.0"
edition = "2021"

[dependencies]
```

#### src/lib.rs

```rust
/// Formats a greeting message with the given name.
pub fn format_greeting(name: &str) -> String {
    let title = determine_title(name);
    format!("Hello, {} {}!", title, name)
}

fn determine_title(name: &str) -> String {
    match name.len() {
        0..=3 => "Sir/Madam".to_string(),
        4..=7 => "Friend".to_string(),
        _ => "Distinguished Guest".to_string(),
    }
}

pub fn create_personalized_greeting(name: String, enthusiasm_level: Option<u32>) -> String {
    let base_greeting = format_greeting(&name);

    let exclamation_marks = match enthusiasm_level {
        Some(level) => "!".repeat(level.min(5) as usize),
        None => "".to_string(),
    };

    format!("{}{}", base_greeting, exclamation_marks)
}

pub fn count_greeting_chars(greeting: &str) -> usize {
    greeting.chars().count()
}

pub fn is_valid_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_alphabetic() || c.is_whitespace())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_greeting() {
        assert_eq!(format_greeting("Bob"), "Hello, Sir/Madam Bob!");
        assert_eq!(format_greeting("Alice"), "Hello, Friend Alice!");
    }
}
```

#### src/main.rs

```rust
use std::env;
use oxur_syntax_exploration::{
    create_personalized_greeting,
    format_greeting,
    count_greeting_chars,
    is_valid_name,
};

fn main() {
    let args: Vec<String> = env::args().collect();

    let name = if args.len() > 1 {
        args[1].clone()
    } else {
        String::from("World")
    };

    if !is_valid_name(&name) {
        eprintln!("Error: '{}' is not a valid name", name);
        std::process::exit(1);
    }

    let enthusiasm = if args.len() > 2 {
        args[2].parse::<u32>().ok()
    } else {
        Some(1)
    };

    let simple_greeting = format_greeting(&name);
    println!("Simple greeting: {}", simple_greeting);

    let personalized = create_personalized_greeting(name.clone(), enthusiasm);
    println!("Personalized greeting: {}", personalized);

    let char_count = count_greeting_chars(&personalized);
    println!("Character count: {}", char_count);

    println!("\nOriginal name provided: {}", name);

    print_usage_info();
}

fn print_usage_info() {
    println!("\nUsage: oxur_syntax_exploration [NAME] [ENTHUSIASM_LEVEL]");
    println!("  NAME: The name to greet (default: World)");
    println!("  ENTHUSIASM_LEVEL: Number 0-5 for excitement (default: 1)");
}
```

### Appendix B-G: Lisp Translations

Due to length, please refer to the individual source files in the project:

- `main.rkt` - Racket (dynamic)
- `main-typed.rkt` - Typed Racket
- `main.shen` - Shen
- `main.coalton` - Coalton
- `oxur-greeting-lfe.lfe` - LFE
- `oxur-greeting-clojure.clj` - Clojure with core.typed

### Appendix H: Complete Oxur Example

See `oxur-comprehensive.oxur` for a complete showcase including:

- Module system
- Structs and enums
- Traits and implementations
- Pattern matching
- Generics and lifetimes
- Error handling
- Collections and iterators
- Closures and higher-order functions
- Macros
- Concurrency primitives

---

## Conclusion

Oxur represents a synthesis of the best ideas from Lisp and Rust. By choosing inline type annotations, maintaining Rust's ownership semantics, and embracing Lisp's s-expression syntax, we create a language that is:

- **Safe** - Full Rust semantics, compile-time guarantees
- **Expressive** - Lisp macros and homoiconicity
- **Pragmatic** - Types where they matter, not ceremony
- **Familiar** - Readable to both Rust and Lisp programmers
- **Consistent** - Logical naming (`deffn`, not `defun`)

The journey from exploring multiple Lisp dialects to discovering the `name:type` syntax has led us to a design that feels both **natural** and **innovative**.

**Oxur is a Lisp that doesn't compromise on safety, and a systems language that doesn't compromise on expressiveness.**

---

*Document Version: 1.0*
*Date: January 1, 2025*
*Status: Design Proposal*

**Next Steps:**

1. Implementation planning
2. Parser/lexer design
3. AST representation
4. Rust code generation strategy
5. REPL implementation
6. Standard library design
7. Tooling development

**Happy New Year, and happy hacking!** 🎉🦀✨
