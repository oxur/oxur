;; Test: Block with multiple statements

(Block
  :stmts ((Stmt
    :kind (Semi
      :expr (Expr
        :kind (Path :segments ("first"))
        :span (Span :lo 0 :hi 0)))
    :span (Span :lo 0 :hi 0))
   (Stmt
    :kind (Expr
      :expr (Expr
        :kind (Path :segments ("second"))
        :span (Span :lo 0 :hi 0)))
    :span (Span :lo 0 :hi 0)))
  :span (Span :lo 0 :hi 0))
