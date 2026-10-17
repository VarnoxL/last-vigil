# Rumors are structured claims with model-written wording

A rumor is a structured claim that Rust tracks exactly (source, confidence, hops, truth link), and the model only rewrites its wording at each hop. Pure free-text rumors can't be traced or scored against the hidden truth, and pure structured rumors read like a spreadsheet. The hybrid keeps the ancestry exact while the text drifts organically.

**Considered options:** free-text only (untraceable), structured only (lifeless).
