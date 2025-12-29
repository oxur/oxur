(Item
  :vis (Inherited)
  :ident (Ident :name "foo")
  :kind (Fn
          (Fn :defaultness Final
          :sig (FnSig
                 :header (FnHeader :safety Default :constness NotConst)
                 :decl (FnDecl :inputs () :output (Default)))
          :generics (Generics :params ())
          :body nil)))
