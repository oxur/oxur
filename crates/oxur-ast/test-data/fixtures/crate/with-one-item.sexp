;; Test: Crate with a single item

(Crate
  :items ((Item
    :vis (Public)
    :ident (Ident :name "main")
    :kind (Fn
      :defaultness Final
      :sig (FnSig
        :header (FnHeader :safety Default :constness NotConst)
        :decl (FnDecl :inputs () :output (Default)))
      :generics (Generics :params ())
      :body (Block :stmts () :id 1 :span (Span :lo 0 :hi 0)))))
  :spans (ModSpans :inner-span (Span :lo 0 :hi 0))
  :id 0)
