;; Test: Immutable reference type (&T)
(Item
  :ident (Ident :name "REF_TYPE")
  :kind (Const :ty (Ty :kind (Ref :ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "i32"))))))))))
