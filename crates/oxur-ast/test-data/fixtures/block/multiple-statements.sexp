(Block
  :stmts ((Stmt :id 1 :kind (Empty) :span (Span))
          (Stmt :id 2 :kind (Semi (Expr
                                      :id 3
                                      :kind (MacCall
                          (MacCall :path (Path :segments ((PathSegment :ident (Ident :name "test"))))))))
                :span (Span))
          (Stmt :id 4 :kind (Expr (Expr
                                      :id 5
                                      :kind (MacCall
                          (MacCall :path (Path :segments ((PathSegment :ident (Ident :name "final"))))))))
                :span (Span)))
  :id 10
  :span (Span :lo 0 :hi 100))
