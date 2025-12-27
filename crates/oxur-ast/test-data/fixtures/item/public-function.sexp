;; Test: Public function item

(Item
  :vis Public
  :kind (Fn
    :sig (FnSig :name "foo" :params () :return-type nil)
    :body (Block :stmts () :span (Span :lo 0 :hi 0)))
  :span (Span :lo 0 :hi 0))
