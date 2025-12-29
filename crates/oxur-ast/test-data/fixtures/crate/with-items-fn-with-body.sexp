(Crate
  :attrs ()
  :items ((Item
            :vis (Inherited)
            :ident (Ident :name "main")
            :kind (Fn
          (Fn :defaultness Final
                    :sig (FnSig
                           :header (FnHeader
                                     :safety Default
                                     :constness NotConst)
                           :decl (FnDecl
                                   :inputs ()
                                   :output (Default)))
                    :generics (Generics :params () :where-clause (WhereClause :has-where-token false :predicates ()))
                    :body nil))))
  :spans (ModSpans :inner-span (Span :lo 0 :hi 50))
  :id 0)
