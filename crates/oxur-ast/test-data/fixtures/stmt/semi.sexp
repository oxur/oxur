;; Test: Semicolon statement with expression

(Stmt
  :kind (Semi
    :expr (Expr
      :kind (Path :segments ("value"))
      :span (Span :lo 0 :hi 0)))
  :span (Span :lo 0 :hi 0))
