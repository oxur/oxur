;; Test: Array type [T; N]
(Item
  :ident (Ident :name "ARRAY_TYPE")
  :kind (Const :ty (Ty :kind (Array
    :ty (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "i32"))))))
    :len (Expr :id 1 :kind (Lit (Lit :kind (Int 10))))))))
