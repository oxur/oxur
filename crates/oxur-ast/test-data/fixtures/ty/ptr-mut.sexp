;; Test: Mut raw pointer type (*mut T)
(Item
  :ident (Ident :name "PTR_MUT_TYPE")
  :kind (Const :ty (Ty :kind (Ptr :mutability Mut :ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "i32"))))))))))
