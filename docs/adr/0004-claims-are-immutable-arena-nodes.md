# Claims are immutable per-hop nodes in an arena

Every hop creates a new `Claim` that is never changed afterward. It is stored in `RumorEngine::claims: Vec<Claim>` with its index as its id, and it points back to its `parent` and `root`. Ancestry is then just following parent links, a claim told to many people naturally forms a tree, and the dump is the arena serialized as-is. Index ids also keep the borrow checker out of the way: there are no `Rc<RefCell<…>>` graphs, and engine state stays a `Vec`, which keeps the determinism fingerprint ordered (ADR 0003). Nodes are never removed, so the arena only grows. At the town's scale that's fine; if Long runs make it too big, compact dead roots rather than switching to mutable claims.

**Considered options:** one shared mutable claim per rumor with a separate hop log (two sources of truth for confidence), a pointer graph with `Rc<RefCell<Claim>>` (fights the borrow checker, `Debug` can't print cycles, and it doesn't serialize cleanly).
