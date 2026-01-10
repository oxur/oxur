;; Test: Invalid type symbol (should fail)
(Item
  :ident (Ident :name "INVALID_SYMBOL_TYPE")
  :kind (Const :ty (Ty :kind InvalidSymbol)))
