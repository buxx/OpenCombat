# Open Combat roadmap

## Battle

### Soldiers

- [x] Basic soldier behavior (moves, defend, hide, etc.)
- [x] Movement modes (walk, run, crawl/sneak) with speed by behavior and terrain
- [x] Basic autonomous decisions (engage nearest visible enemy, switch target, move fast when enemies are visible)
- [ ] Evolved autonomous decisions (engage nearest visible enemy, switch target, move fast when enemies are visible)
- [x] Automatic grenade throw when close to an enemy
- [ ] Evolved soldier behavior (flank, retreat, etc)
- [ ] Soldiers seek cover by themselves when under fire / moving
- [ ] Rout & surrender (soldiers fleeing the battlefield or giving up)
- [x] Ambush order (hold fire until enemy is close)
- [x] Basic status (operational or dead)
- [x] Stunned status (from explosions)
- [x] Supress (alter aming, relading, behavior, etc)
- [x] Postures (standing, prone, crawling) and gestures (aiming, reloading)
- [x] Experience (affects aiming time, accuracy, grenade scatter)
- [ ] Experience gain over battles, veterans vs. green troops behaviors (obedience, panic threshold)
- [ ] Fatigue (run, weight of equipment, etc): rested / winded / fatigued
- [ ] Evolved status (panic, injured, etc.)
- [ ] Individual morale/stress (casualties nearby, leader killed, flanked, success reduces stress)
- [ ] Wounded / incapacitated soldiers (slower, reduced fire, can't act)
- [x] Weapons, magazines & ammunition (rifles, machine guns with bursts, reloading)
- [ ] Weapon bust mode choice according to situation / placement
- [ ] Out of ammo handling (scavenge ammo/weapons from dead soldiers, bayonet)
- [ ] Secondary weapons / fire mode choice
- [x] Grenades
- [x] Smokes
- [ ] Close combat fight (knife/hand/revolver)
- [ ] Flamethrower
- [ ] Satchel charges / engineers (demolish walls, clear obstacles)
- [ ] Snipers
- [ ] Mortar
- [ ] AT
- [ ] Setup weapons (mortar, heyvy machine guns)

### Squads & Command

- [x] Squad leader (orders go through leader, new leader when he dies)
- [x] Squad movement by formation (simple positions)
- [x] Fire positions (members spread to nearby tiles allowing fire, preferring cover)
- [x] Chained orders (waypoints, drag order markers)
- [x] Chained orders bis (different orders type chain)
- [ ] Evolved squad movement (fastest/best covering path)
- [ ] Multiple formations (column, wedge, etc.)
- [ ] Command hierarchy (platoon/company leaders, command radius improving morale and effectiveness)
- [ ] Radio / communication delay for orders

### Vehicles

- [ ] Basic vehicle (one tank, one light vehicle)
- [ ] Enhanced vehicle (realistic blindage and AT penetration, more vehicles types)
- [ ] Vehicle damage states (immobilized, gun damaged, crew bail out, abandoned)
- [ ] Vehicle crew (crew members as soldiers, re-crew abandoned vehicles)
- [ ] Transport (infantry mount/dismount)
- [ ] Hull down, flank and rear armor
- [ ] Towable guns

### Support

- [x] Off-map naval bombardment (limited uses, preparation delay, salvos, cooldown)
- [ ] More off-map artillery / mortar barrages / Air strikes
- [ ] Paratroopers
- [ ] Smoke barrages

### Physics & World

- [x] Tiles specificities (opacity, movement alteration, hiding bonus, bullet interception)
- [x] Tiles alteration (on destruction) with new properties (opacity, movement alteration, hiding bonus, bullet interception)
- [ ] Tiles alteration with z modification (craters) 
- [x] Simple 3D (square at precise height)
- [ ] Better 3D (no more cubic sides)
- [x] Bullet ballistics in 3D (height, walls and hills block bullets)
- [x] Explosions (kill, stun, suppress, shrapnel and destruction radius; building protection)
- [x] Smoke effect 
- [x] Line of sight and fog of war (enemies visible only when seen)
- [ ] Last seen soldiers markers and targets
- [x] Building interiors (visible when a soldier is inside)
- [ ] Obstacles (barbed wire, mines, tank traps)
- [ ] Destroyable bridges
- [ ] Weather and time of day (night, rain, fog)

### Decor & Display & Fx

- [x] Trees draw on the top of soldiers, big bushes, etc.
- [x] Craters
- [x] Debris on destruction (wall, houses)
- [x] Soldier animations and weapon sprites
- [x] Muzzle flashes, bullet tracers
- [x] Base explosion effects (animation, light flash)
- [x] Evolved explosion effects (more animations, particles, smoke puffs)
- [x] Basic sounds (gunshot, soldier hit, explosion, impacts)
- [ ] Sound enhancement (more sounds and more realistic sounds)
- [ ] Soldier voices (shouts, orders acknowledgement, wounded calls)
- [ ] Wounded/hurt animations
- [ ] Fire / burning (buildings, vehicles)
- [ ] Topographic map

### UI & Controls

- [x] Squad order menu
- [x] Order previews (paths, line of sight, throw range, direction markers)
- [x] Line of sight tool
- [x] World map overview and 3D relief view (F2)
- [x] Hide decor (T)
- [x] Minimap
- [x] Basic HUD: squads, soldiers (portrait, status, ammo, grenades), morale bar, remaining time, support buttons
- [x] Enhanced Basic HUD: more informations, icons, etc
- [ ] Keyboard shortcuts for orders
- [ ] Soldier details panel (status, morale, fatigue, experience, weapons)
- [ ] End of battle result screen (casualties, kills, flags, per soldier stats)
- [ ] Surrender

### Immersion

- [ ] Realistic weapons and equipement list
- [ ] Soldier names and photographies
- [ ] Real scenarios with terrain and dates
- [ ] Medals / awards
- [ ] Historical briefings
- [ ] Historic battles and campaigns

### Game

- [x] "One" battle phase (no continue battle)
- [x] Deployment phase
- [x] Flags
- [ ] Flags count for victory
- [x] End of battle by time limit or morale
- [ ] Army morale taking flags, leaders and events into account (not only casualties)
- [ ] Cease fire (both sides agree to end battle)
- [x] Battle scenarios (description, deployment profiles, squads count, spawn zones)
- [ ] Reinforcements during battle
- [ ] Replays
- [ ] Persistent soldiers for "continue" battle/campaign (casualties, experience, fatigue carried over battles)
- [ ] Strategic map / Campaign (move battle groups, assign supports, resolve battles on tactical maps)

### AI

- [ ] Basic AI opponent
- [ ] AI difficulty and realism settings

### Modding & Tools

- [x] Mods capable
- [x] Basic map editor (Tiled)
- [ ] Specialized map editor

### Multiplayer

- [x] Multiplayer capable architecture
- [ ] Multiplayer through GUI instead complex CLIs
- [ ] More than 2 players (team play, several players per side)
- [ ] Enhanced multiplayer (more user friendly, password management, firewall management, etc)
