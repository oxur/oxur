;; Visibility variants
;; Shows a crate with items having different visibility levels
;;
;; Demonstrates Public, Inherited, and other visibility modifiers.

(Crate
  :items ((Item
    :vis Public
    :kind (Fn
      :sig (FnSig :name "public_fn" :params () :return-type nil)
      :body (Block :stmts () :span (Span :lo 0 :hi 0)))
    :span (Span :lo 0 :hi 0))
   (Item
    :vis Inherited
    :kind (Fn
      :sig (FnSig :name "private_fn" :params () :return-type nil)
      :body (Block :stmts () :span (Span :lo 0 :hi 0)))
    :span (Span :lo 0 :hi 0)))
  :span (Span :lo 0 :hi 0))
