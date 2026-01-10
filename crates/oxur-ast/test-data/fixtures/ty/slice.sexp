;; Test: Slice type [T]
(Item
  :ident (Ident :name "SLICE_TYPE")
  :kind (Const :ty (Ty :kind (Slice (Ty :kind (Path nil (Path :segments ((PathSegment :ident (Ident :name "i32"))))))))))
