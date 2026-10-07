variable (P Q : Prop)

theorem modus_ponens (h1 : P → Q) (h2 : P) : Q := by
  apply h1
  exact h2
