;; Test: Mutable reference type (&mut T)
(Item
  :ident (Ident :name "REF_MUT_TYPE")
  :kind (Const :ty (Ty :kind (Ref :mutability Mut :ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "i32"))))))))))
