;; Test: Function with non-empty body

(Item
  :vis Public
  :kind (Fn
    :sig (FnSig :name "with_body" :params () :return-type nil)
    :body (Block
      :stmts ((Stmt
        :kind (Semi
          :expr (Expr
            :kind (Path :segments ("value"))
            :span (Span :lo 0 :hi 0)))
        :span (Span :lo 0 :hi 0)))
      :span (Span :lo 0 :hi 0)))
  :span (Span :lo 0 :hi 0))
