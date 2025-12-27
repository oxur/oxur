;; Full crate with multiple items
;; A realistic crate containing multiple function definitions
;;
;; This demonstrates a complete crate structure with:
;; - Public and private functions
;; - Function with statements in body
;; - Multiple items in a single crate

(Crate
  :items ((Item
    :vis Public
    :kind (Fn
      :sig (FnSig
        :name "main"
        :params ()
        :return-type nil)
      :body (Block
        :stmts ((Stmt
          :kind (Semi
            :expr (Expr
              :kind (MacCall
                :path (Path :segments ("println"))
                :delim Paren
                :tokens ("\"Hello, world!\""))
              :span (Span :lo 20 :hi 45)))
          :span (Span :lo 20 :hi 46)))
        :span (Span :lo 15 :hi 48)))
    :span (Span :lo 0 :hi 48))
   (Item
    :vis Inherited
    :kind (Fn
      :sig (FnSig
        :name "helper"
        :params ()
        :return-type nil)
      :body (Block
        :stmts ()
        :span (Span :lo 70 :hi 72)))
    :span (Span :lo 50 :hi 72)))
  :span (Span :lo 0 :hi 72))
