;; Function with multiple statements
;; Shows a realistic function body with several operations
;;
;; Demonstrates:
;; - Multiple statements in a block
;; - Mix of expression and semi statements
;; - Proper statement ordering

(Item
  :vis Public
  :kind (Fn
    :sig (FnSig
      :name "process"
      :params ()
      :return-type nil)
    :body (Block
      :stmts ((Stmt
        :kind (Semi
          :expr (Expr
            :kind (MacCall
              :path (Path :segments ("println"))
              :delim Paren
              :tokens ("\"Starting\""))
            :span (Span :lo 0 :hi 20)))
        :span (Span :lo 0 :hi 21))
       (Stmt
        :kind (Semi
          :expr (Expr
            :kind (Path
              :segments ("do_work"))
            :span (Span :lo 25 :hi 33)))
        :span (Span :lo 25 :hi 34))
       (Stmt
        :kind (Semi
          :expr (Expr
            :kind (MacCall
              :path (Path :segments ("println"))
              :delim Paren
              :tokens ("\"Done\""))
            :span (Span :lo 38 :hi 55)))
        :span (Span :lo 38 :hi 56)))
      :span (Span :lo 0 :hi 60)))
  :span (Span :lo 0 :hi 60))
