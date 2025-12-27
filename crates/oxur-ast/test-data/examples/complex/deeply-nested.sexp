;; Deeply nested structure
;; Stress test for nested blocks and expressions
;;
;; This tests the parser's ability to handle deep nesting
;; and maintain correct structure throughout.

(Crate
  :items ((Item
    :vis Public
    :kind (Fn
      :sig (FnSig
        :name "nested"
        :params ()
        :return-type nil)
      :body (Block
        :stmts ((Stmt
          :kind (Expr
            :expr (Expr
              :kind (Block
                :block (Block
                  :stmts ((Stmt
                    :kind (Expr
                      :expr (Expr
                        :kind (Block
                          :block (Block
                            :stmts ((Stmt
                              :kind (Semi
                                :expr (Expr
                                  :kind (Path
                                    :segments ("value"))
                                  :span (Span :lo 0 :hi 0)))
                              :span (Span :lo 0 :hi 0)))
                            :span (Span :lo 0 :hi 0)))
                        :span (Span :lo 0 :hi 0)))
                    :span (Span :lo 0 :hi 0)))
                  :span (Span :lo 0 :hi 0)))
              :span (Span :lo 0 :hi 0)))
          :span (Span :lo 0 :hi 0)))
        :span (Span :lo 0 :hi 0)))
    :span (Span :lo 0 :hi 0)))
  :span (Span :lo 0 :hi 0))
