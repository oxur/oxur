;; Test: Crate with multiple items

(Crate
  :items ((Item
    :vis Public
    :kind (Fn
      :sig (FnSig :name "first" :params () :return-type nil)
      :body (Block :stmts () :span (Span :lo 0 :hi 0)))
    :span (Span :lo 0 :hi 0))
   (Item
    :vis Inherited
    :kind (Fn
      :sig (FnSig :name "second" :params () :return-type nil)
      :body (Block :stmts () :span (Span :lo 0 :hi 0)))
    :span (Span :lo 0 :hi 0)))
  :span (Span :lo 0 :hi 0))
