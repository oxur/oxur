;; Comprehensive example with many node types
;; Exercises multiple AST node variants
;;
;; This file demonstrates many different node types:
;; - Crate, Item, Fn, FnSig, Block, Stmt, Expr
;; - Various visibility levels
;; - Different expression kinds
;; - Statement variations

(Crate
  :items ((Item
    :vis Public
    :kind (Fn
      :sig (FnSig
        :name "example"
        :params ()
        :return-type nil)
      :body (Block
        :stmts ((Stmt
          :kind (Semi
            :expr (Expr
              :kind (Path
                :segments ("std" "io" "Result"))
              :span (Span :lo 0 :hi 14)))
          :span (Span :lo 0 :hi 15))
         (Stmt
          :kind (Semi
            :expr (Expr
              :kind (MacCall
                :path (Path :segments ("vec"))
                :delim Bracket
                :tokens ("1" "," "2" "," "3"))
              :span (Span :lo 20 :hi 35)))
          :span (Span :lo 20 :hi 36))
         (Stmt
          :kind (Expr
            :expr (Expr
              :kind (Path
                :segments ("result"))
              :span (Span :lo 40 :hi 46)))
          :span (Span :lo 40 :hi 46)))
        :span (Span :lo 0 :hi 50)))
    :span (Span :lo 0 :hi 50))
   (Item
    :vis Inherited
    :kind (Fn
      :sig (FnSig
        :name "private_helper"
        :params ()
        :return-type nil)
      :body (Block
        :stmts ()
        :span (Span :lo 70 :hi 72)))
    :span (Span :lo 60 :hi 72)))
  :span (Span :lo 0 :hi 72))
