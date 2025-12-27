;; Test: Const function item

(Item
  :vis Public
  :kind (Fn
    :sig (FnSig :name "const_fn" :params () :return-type nil :const true)
    :body (Block :stmts () :span (Span :lo 0 :hi 0)))
  :span (Span :lo 0 :hi 0))
