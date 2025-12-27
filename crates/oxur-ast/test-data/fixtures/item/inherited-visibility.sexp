;; Test: Inherited (private) visibility function

(Item
  :vis Inherited
  :kind (Fn
    :sig (FnSig :name "private_fn" :params () :return-type nil)
    :body (Block :stmts () :span (Span :lo 0 :hi 0)))
  :span (Span :lo 0 :hi 0))
