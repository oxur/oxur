;; Test: Bare function type with ABI extern "C" fn()
(Item
  :ident (Ident :name "BARE_FN_ABI_TYPE")
  :kind (Const :ty (Ty :kind (BareFn :abi "C"))))
