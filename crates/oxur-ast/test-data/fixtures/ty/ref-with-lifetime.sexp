;; Test: Reference with lifetime (&'a T)
(Item
  :ident (Ident :name "REF_LIFETIME_TYPE")
  :kind (Const :ty (Ty :kind (Ref
    :lifetime (Lifetime :ident (Ident :name "a"))
    :ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "i32"))))))))))
