# Last Vigil

A haunted medieval town under a doomsday prophecy, simulated as a society of agents who know only what they have seen or been told.

## The world

**Hidden truth**:
What is actually going on in a run (fraud prophet, hoarder, miscalculated date, real doom), generated from the seed. No agent ever sees it directly.
_Avoid_: secret, answer, plot

**Truth fragment**:
One piece of the hidden truth, held by a ghost.
_Avoid_: clue, hint

**Countdown**:
The town-wide clock to the prophesied doom. Every agent can see it.
_Avoid_: timer, doomsday clock (in UI text)

**Run**:
One playthrough, from seed to countdown zero.
_Avoid_: game, session, episode

**Run length**:
The preset (Quick, Standard, Long) that sets how many countdown days a run has and how much real time each day takes.
_Avoid_: speed, difficulty

**Replay**:
A run played back from its replay log, with no models called.
_Avoid_: recording, rerun

## Information

**Claim**:
A structured statement about the town (what it says, who told it, who heard it, when, how confident the teller was) and whether it matches the hidden truth. Each hop produces a new claim that remembers the claim it came from, so one claim can be told to many people and branch.
_Avoid_: rumor (for the data), fact, message

**Rumor**:
A claim as a person in town would say it: the reworded text. One rumor belongs to one claim.
_Avoid_: gossip, story

**Hop**:
One retelling of a claim from one agent to another. Each hop creates a new claim.
_Avoid_: step, jump, generation

**Hearing range**:
How close two agents must be, and in the same room, for a hop to be possible. Walls block it.
_Avoid_: radius, proximity

**Plant**:
Putting a claim into an agent from outside the town's own talk (the player's whisper, a ghost's omen, or a test).
_Avoid_: inject, seed (a seed means the run's random seed)

**Sighting**:
A claim an agent forms by seeing something happen with their own eyes, not by being told.
_Avoid_: observation, witness (as a noun)

**Corroboration**:
Hearing the same thing from a source with a different ancestry. It raises the influence of the agent who said it first.
_Avoid_: confirmation, proof

**Ancestry**:
The chain of claims from a claim back to where it started, which is a plant or a sighting. Branches form a tree.
_Avoid_: history, lineage, trace (as a noun for the data)

**Omen**:
A cryptic sign from a ghost, such as a dream, a flicker, or a medium's words. It enters town as a low-clarity claim.
_Avoid_: message, hint

**Whisper**:
A claim planted by the player in the god role.
_Avoid_: injection, prompt

## Agents

**Tier**:
How much thinking an agent gets: T0 crowd (no model), T1 townsfolk (small model, event-driven), T2 key figure (large model, memory and reflection).
_Avoid_: level, class, rank

**Promotion / Demotion**:
An agent moving up or down a tier mid-run, depending on how involved it is in the story.
_Avoid_: upgrade, escalation

**Station**:
An agent's place in town (prophet, priest, tavern keeper, beggar, and so on), set at run start.
_Avoid_: role (that's the player's), job, class

**Influence**:
How much weight an agent's words carry: how often they speak, how readily others believe them, and how long their claims stay fresh. It starts from their station and shifts with their state and track record.
_Avoid_: status, reputation, charisma

**Bark**:
A canned line spoken by a T0 agent.
_Avoid_: dialogue, line

**Ghost**:
A dead townsperson who holds truth fragments and can speak only through omens.
_Avoid_: spirit, NPC

**Medium**:
A living agent through whom a ghost can speak.
_Avoid_: channeler, psychic

**Role**:
How the player takes part: Observer, God, Cult Member, or Investigator.
_Avoid_: mode, character, class
