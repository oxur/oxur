;; Test: Bare function type with named parameter fn(x: i32)
(Item
  :ident (Ident :name "BARE_FN_NAMED_PARAM_TYPE")
  :kind (Const :ty (Ty :kind (BareFn
    :inputs ((BareFnParam
      :name (Ident :name "x")
      :ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "i32"))))))))))))
