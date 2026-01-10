;; Test: Invalid type kind (should fail)
(Item
  :ident (Ident :name "INVALID_TYPE")
  :kind (Const :ty (Ty :kind (InvalidKind))))
