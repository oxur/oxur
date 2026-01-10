;; Test: Macro call type my_type!()
(Item
  :ident (Ident :name "MAC_CALL_TYPE")
  :kind (Const :ty (Ty :kind (MacCall :mac (MacCall :path (Path :segments ((PathSegment :ident (Ident :name "my_type")))))))))
