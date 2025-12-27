;; Test: Block containing nested block expression

(Block
  :stmts ((Stmt
    :kind (Expr
      :expr (Expr
        :kind (Block
          :block (Block
            :stmts ()
            :span (Span :lo 0 :hi 0)))
        :span (Span :lo 0 :hi 0)))
    :span (Span :lo 0 :hi 0)))
  :span (Span :lo 0 :hi 0))
