;; Test: Block expression

(Expr
  :kind (Block
    :block (Block
      :stmts ((Stmt
        :kind (Semi
          :expr (Expr
            :kind (Path :segments ("x"))
            :span (Span :lo 0 :hi 0)))
        :span (Span :lo 0 :hi 0)))
      :span (Span :lo 0 :hi 0)))
  :span (Span :lo 0 :hi 0))
