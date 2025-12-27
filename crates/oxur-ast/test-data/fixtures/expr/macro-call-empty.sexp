;; Test: Macro call with empty arguments

(Expr
  :kind (MacCall
    :path (Path :segments ("vec"))
    :delim Bracket
    :tokens ())
  :span (Span :lo 0 :hi 0))
