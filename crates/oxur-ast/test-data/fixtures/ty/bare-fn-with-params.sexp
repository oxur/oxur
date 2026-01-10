;; Test: Bare function type with parameters fn(i32)
(Item
  :ident (Ident :name "BARE_FN_PARAMS_TYPE")
  :kind (Const :ty (Ty :kind (BareFn
    :inputs ((BareFnParam :ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "i32"))))))))))))
