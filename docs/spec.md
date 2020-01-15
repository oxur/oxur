# An Informal Oxur Specification

## Lexical Structure

### Comments

```rust
// Comment
```

```lisp
;;; Module-level comment
;; Code-block-levelm comment
; End-of-line-level comment
```

TODO: Doc comments

### Identifiers

Legal characters:
* In Common Lisp: http://www.lispworks.com/documentation/HyperSpec/Body/02_ac.htm
* In Scheme R6RS: http://www.r6rs.org/final/html/r6rs/r6rs-Z-H-7.html#node_sec_4.2.4
* In Clojure: https://clojure.org/reference/reader#_symbols

Characters not supported by Rust will be converted to characters that are supported.

### Literals

## Items

### Mods

```rust
mod math {

    fn sin(f: f64) -> f64 {
        /* ... */
    }

}
```

```lisp
(ns math)

(defn ^f64 sin
  [^f64 f]
  ;;
  )
```

### Extern Crates

```rust
extern crate pcre;
extern crate std;
extern crate std as ruststd;
extern crate foo as _;
```

```lisp
(extern
  (:crate
    [pcrs]
    [std]
    [std :as ruststd]
    [foo :as _]))
```

### Imports

```rust
use std::option::Option::{Some, None};
use std::collections::hash_map::{self, HashMap};
use self::foo::Zoo as _;
use quux::*;
```

```lisp
(ns
  (:require
    [code.written.in.oxur :as oxur])
  (:import
    (std.option.Option (Some None))
    (std.collections.hash-map (self HashMap))
    (self.foo.Zoo :as _)
    (quux :refer :all))
```

### Functions

```rust
fn main() {
    println!("Hello, world!");
}
```

```lisp
(defn main []
  (println! "Hello, world!"))
```

TODO: args
TODO: types
TODO: patterns
TODO: generic functions
TODO: Extern function qualifier support
TODO: const functions
TODO: async, async unsafe functions
TODO: function attributes
TODO: function parameter attributes
