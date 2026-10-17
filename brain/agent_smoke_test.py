"""Phase 1 smoke test: do Tier 1 / Tier 2 agents work on this GPU?

Checks, in order:
  1. Ollama is reachable and both models are pulled.
  2. A T1 (small) and T2 (large) agent each hear the same rumor and must return a
     schema-valid decision: believe?, repeat?, rewritten wording.
  3. Latency and tokens/sec per call.
  4. Both models resident at once (co-residency) and VRAM used, via nvidia-smi.

Run:  python brain/agent_smoke_test.py
Needs only the stdlib and a running Ollama.
"""
import json
import subprocess
import sys
import time
import urllib.request

OLLAMA = "http://localhost:11434"
T1_MODEL = "llama3.2:3b"
T2_MODEL = "qwen3:8b"

DECISION_SCHEMA = {
    "type": "object",
    "properties": {
        "believes": {"type": "boolean"},
        "belief_strength": {"type": "number", "minimum": 0, "maximum": 1},
        "will_repeat": {"type": "boolean"},
        "rewritten": {"type": "string"},
        "inner_thought": {"type": "string"},
    },
    "required": ["believes", "belief_strength", "will_repeat", "rewritten", "inner_thought"],
}

RUMOR = {
    "claim": "The prophet slips out the east gate at midnight.",
    "source": "Marta, a baker",
    "confidence": 0.78,
    "hops": 1,
}

AGENTS = {
    "T1": (T1_MODEL, "Odo, a nervous farmhand. Devout but easily frightened. Distrusts outsiders."),
    "T2": (T2_MODEL, "Brother Aldric, the prophet's right hand. Proud, secretly doubts the prophecy, "
                     "terrified of losing his standing before the doom in 2 days."),
}


def post(path, body, timeout=600):
    req = urllib.request.Request(OLLAMA + path, json.dumps(body).encode(), {"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return json.load(r)


def get(path):
    with urllib.request.urlopen(OLLAMA + path, timeout=10) as r:
        return json.load(r)


def vram():
    try:
        out = subprocess.check_output(
            ["nvidia-smi", "--query-gpu=memory.used,memory.total", "--format=csv,noheader,nounits"], text=True)
        used, total = (int(x) for x in out.strip().split(","))
        return used, total
    except Exception:
        return None


def decide(model, persona):
    prompt = (
        f"You are {persona}\n"
        f"The doomsday countdown has 2 days left. You just heard a rumor.\n"
        f"Rumor: \"{RUMOR['claim']}\" (told by {RUMOR['source']}, confidence {RUMOR['confidence']}, hop {RUMOR['hops']}).\n"
        f"Decide whether you believe it, whether you will pass it on, and how you would say it in your own words. "
        f"Reply as JSON only."
    )
    t0 = time.time()
    r = post("/api/chat", {
        "model": model,
        "messages": [{"role": "user", "content": prompt}],
        "format": DECISION_SCHEMA,
        "stream": False,
        "think": False,
        "options": {"temperature": 0.8, "num_predict": 300},
    })
    dt = time.time() - t0
    out = json.loads(r["message"]["content"])
    for k in DECISION_SCHEMA["required"]:
        assert k in out, f"missing field {k}"
    toks = r.get("eval_count", 0)
    tps = toks / (r["eval_duration"] / 1e9) if r.get("eval_duration") else 0
    return out, dt, tps


def main():
    try:
        tags = {m["name"] for m in get("/api/tags")["models"]}
    except Exception as e:
        sys.exit(f"FAIL: Ollama not reachable at {OLLAMA}: {e}")
    missing = [m for m in (T1_MODEL, T2_MODEL) if m not in tags]
    if missing:
        sys.exit(f"FAIL: models not pulled yet: {missing}  (ollama pull <name>)")

    base = vram()
    print(f"VRAM before: {base[0]} / {base[1]} MiB" if base else "VRAM: nvidia-smi unavailable")
    ok = True
    for tier, (model, persona) in AGENTS.items():
        print(f"\n--- {tier} agent on {model} ---")
        try:
            out, dt, tps = decide(model, persona)
        except Exception as e:
            ok = False
            print("FAIL:", e)
            continue
        print(f"latency {dt:.1f}s   {tps:.0f} tok/s")
        print(f"believes={out['believes']} strength={out['belief_strength']:.2f} repeat={out['will_repeat']}")
        print(f"rewritten : {out['rewritten']}")
        print(f"thought   : {out['inner_thought']}")

    print("\n--- co-residency ---")
    # warm both, keep them loaded, then see what Ollama says is resident
    for model in (T1_MODEL, T2_MODEL):
        post("/api/generate", {"model": model, "prompt": "hi", "stream": False, "keep_alive": "10m",
                               "options": {"num_predict": 1}})
    ps = get("/api/ps")["models"]
    for m in ps:
        gpu = m.get("size_vram", 0) / 1e9
        tot = m.get("size", 0) / 1e9
        print(f"{m['name']:16} {tot:.1f} GB total, {gpu:.1f} GB on GPU ({100 * gpu / max(tot, 1e-9):.0f}%)")
    after = vram()
    if after:
        print(f"VRAM after both loaded: {after[0]} / {after[1]} MiB")
    both = {m["name"] for m in ps} >= {T1_MODEL, T2_MODEL}
    fully_gpu = all(m.get("size_vram", 0) >= m.get("size", 1) * 0.99 for m in ps)
    print(f"both resident: {both}   fully on GPU: {fully_gpu}")
    print("\nRESULT:", "PASS" if ok else "FAIL")
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
