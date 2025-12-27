;; Nested block structures
;; Shows how blocks can contain statements and be nested
;;
;; Demonstrates the nesting capability of Block nodes.

(Block
  :stmts ((Stmt
    :kind (Semi
      :expr (Expr
        :kind (Block
          :block (Block
            :stmts ()
            :span (Span :lo 0 :hi 0)))
        :span (Span :lo 0 :hi 0)))
    :span (Span :lo 0 :hi 0)))
  :span (Span :lo 0 :hi 0))
