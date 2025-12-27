;; Test: Macro call with delimited arguments

(Expr
  :kind (MacCall
    :path (Path :segments ("println"))
    :delim Paren
    :tokens ("\"Hello\"" "," "42"))
  :span (Span :lo 0 :hi 0))
