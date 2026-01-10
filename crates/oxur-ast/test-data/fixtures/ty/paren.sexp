;; Test: Parenthesized type (T)
(Item
  :ident (Ident :name "PAREN_TYPE")
  :kind (Const :ty (Ty :kind (Paren (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "i32"))))))))))
