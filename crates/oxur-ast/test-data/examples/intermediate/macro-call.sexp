;; Macro invocation
;; Equivalent to: println!("Hello, world!")
;;
;; This demonstrates a MacCall expression with delimited arguments.

(Expr
  :kind (MacCall
    :path (Path :segments ("println"))
    :delim Paren
    :tokens ("\"Hello, world!\""))
  :span (Span :lo 0 :hi 0))
