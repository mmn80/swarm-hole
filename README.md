# swarm hole

Small Vampire Survivors clone prototype.

*Purpose*: developing a RON asset for configuring generic and quasi-independent attributes, skills, and upgrades, featuring hot reloading for fast prototyping.

- changes to the config files apply to the live game, and even to already spawned entities
- attributes are generic and can be reused at multiple skills
- attribute values for each upgrade level can be defined as absolute values, or as relative additive or multiplicative increases, to allow using multiple characters that have different base skill values
- both player and NPC skills are supported

Demo skills implemented: health, XP gathering, health regeneration, melee, laser, swarm.
Demo attributes: max hp, hp/s, speed, range, acceleration, dps, duration, cooldown.
