;; Test: Unsafe function item

(Item
  :vis Public
  :kind (Fn
    :sig (FnSig :name "unsafe_op" :params () :return-type nil :unsafe true)
    :body (Block :stmts () :span (Span :lo 0 :hi 0)))
  :span (Span :lo 0 :hi 0))
