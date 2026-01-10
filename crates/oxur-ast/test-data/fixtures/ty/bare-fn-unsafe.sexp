;; Test: Unsafe bare function type unsafe fn()
(Item
  :ident (Ident :name "BARE_FN_UNSAFE_TYPE")
  :kind (Const :ty (Ty :kind (BareFn :safety Unsafe))))
