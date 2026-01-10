;; Test: Const raw pointer type (*const T)
(Item
  :ident (Ident :name "PTR_CONST_TYPE")
  :kind (Const :ty (Ty :kind (Ptr :ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "i32"))))))))))
