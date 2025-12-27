;; Test: Crate with a single item

(Crate
  :items ((Item
    :vis Public
    :kind (Fn
      :sig (FnSig :name "main" :params () :return-type nil)
      :body (Block :stmts () :span (Span :lo 0 :hi 0)))
    :span (Span :lo 0 :hi 0)))
  :span (Span :lo 0 :hi 0))
