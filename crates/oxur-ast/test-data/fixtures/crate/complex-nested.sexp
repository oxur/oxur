(Crate
  :attrs ()
  :items ((Item
            :vis (Public)
            :ident (Ident :name "main")
            :kind (Fn
                    (Fn
                      :defaultness Final
                      :sig (FnSig
                             :header (FnHeader
                                       :safety Default
                                       :constness NotConst)
                             :decl (FnDecl :inputs () :output (Default)))
                      :generics (Generics :params ())
                      :body (Block
                              :stmts ((Stmt
                                        :id 1
                                        :kind (Empty)
                                        :span (Span)))
                              :id 2)))))
  :spans (ModSpans :inner-span (Span :lo 0 :hi 100))
  :id 0)
