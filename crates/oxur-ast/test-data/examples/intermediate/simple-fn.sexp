;; Simple function definition
;; Equivalent to: pub fn main() {}
;;
;; This shows a basic Item with a function signature and empty body.

(Item
  :vis Public
  :kind (Fn
    :sig (FnSig
      :name "main"
      :params ()
      :return-type nil)
    :body (Block
      :stmts ()
      :span (Span :lo 0 :hi 0)))
  :span (Span :lo 0 :hi 0))
