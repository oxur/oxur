;; Test: Tuple type (T1, T2)
(Item
  :ident (Ident :name "TUPLE_TYPE")
  :kind (Const :ty (Ty :kind (Tuple
    (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "i32"))))))
    (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "f64"))))))))))
