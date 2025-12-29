(Stmt
  :id 100
  :kind (Semi (Expr
                  :id 101
                  :kind (MacCall
                          (MacCall :path (Path
                                  :segments ((PathSegment :ident (Ident :name "println") :id 102))
                                  :span (Span :lo 0 :hi 7))
                          :args (Delimited
                                  :delim Paren
                                  :tokens (TokenStream :source "\"Hello\""))
                  :span (Span :lo 0 :hi 20)))))
  :span (Span :lo 0 :hi 21))
