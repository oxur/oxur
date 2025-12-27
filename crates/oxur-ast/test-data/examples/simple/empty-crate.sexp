;; Empty crate - the minimal AST structure
;; Equivalent to an empty Rust file: (no items)
;;
;; This is the simplest possible Crate node with no items.

(Crate
  :items ()
  :span (Span :lo 0 :hi 0))
