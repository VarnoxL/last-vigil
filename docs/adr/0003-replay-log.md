# Live runs are made reproducible by a replay log, not by deterministic models

Model replies vary in latency and wording, so a seed alone can't reproduce a live run. The sim instead logs every brain reply along with the tick it was applied on, and replay mode reads from that log instead of calling the brain. This only works if replies are applied at tick boundaries in a fixed order and sim state never depends on `HashMap` iteration order. Both rules look like needless strictness unless you know why they exist, so don't relax them.

**Considered options:** stub-only determinism (can't revisit a good live run), world snapshots / save-resume only (pauses a run but can't replay one).
