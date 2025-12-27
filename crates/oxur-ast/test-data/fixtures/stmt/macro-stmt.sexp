;; Test: Statement containing a macro call

(Stmt
  :kind (Semi
    :expr (Expr
      :kind (MacCall
        :path (Path :segments ("println"))
        :delim Paren
        :tokens ("\"test\""))
      :span (Span :lo 0 :hi 0)))
  :span (Span :lo 0 :hi 0))
