;; Test: Type with explicit ID and span
(Item
  :ident (Ident :name "WITH_ID_SPAN")
  :kind (Const :ty (Ty :id 99 :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "i32"))))) :span (Span :lo 5 :hi 10))))
