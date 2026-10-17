# Three processes; only the brain talks to models

The sim (Rust), brain (Python), and client (TypeScript) run as separate processes over local sockets, and Rust never calls a model directly. Rust gives a deterministic, fast tick loop, Python has the ML tooling, and the browser is where Three.js lives. Keeping every prompt in Python means the sim can be tested and replayed with the brain stubbed or swapped out. Each language was also chosen on purpose, because learning all three is a project goal.
