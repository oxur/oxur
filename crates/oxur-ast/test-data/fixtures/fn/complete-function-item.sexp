(Item
  :id 100
  :vis (Public)
  :ident (Ident :name "calculate" :span (Span :lo 0 :hi 9))
  :kind (Fn
          (Fn
            :defaultness Final
            :sig (FnSig
                   :header (FnHeader :safety Default :constness NotConst)
                   :decl (FnDecl
                           :inputs ((Param) (Param))
                           :output (Default))
                   :span (Span :lo 10 :hi 40))
            :generics (Generics :params ())
            :body (Block
                    :stmts ((Stmt :id 101 :kind (Empty) :span (Span)))
                    :id 102
                    :span (Span :lo 40 :hi 50))))
  :span (Span :lo 0 :hi 50))
