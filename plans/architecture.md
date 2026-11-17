# Plan : architecture du Flappy Bird inspirée de Horizon

Statut : **proposition, rien n'est implémenté**. À valider avant la première ligne de code.

Référence : `horizon-rust/horizon` (Minecraft Java réécrit en Rust, ~80 crates). On reprend ses
**principes**, pas sa taille.

## Ce que Horizon fait et qu'on reprend

| Principe Horizon | Ce qu'il fait chez eux | Version Flappy Bird |
|---|---|---|
| Workspace multi-crates (`crates/`) | ~80 crates, une par responsabilité | 4–5 crates seulement |
| Règle des couches (`docs/layers.md`) | engine → platform → Horizon, vérifié en CI (`check_layers.py`) | `core` (pur) ← `game` ← `client` ; le core ne dépend de rien de graphique |
| Points d'extension plutôt que dépendances montantes | trait / callbacks / registry | traits `Input`, `Clock`, `Rng`, `Storage` injectés dans le core |
| Crates « portables » | logique jouable sans le client desktop | le core tourne en headless → tests, bots, replay, futur port WASM |
| Lints workspace (`[workspace.lints]`) + `clippy -D warnings` | partagés par toutes les crates | idem + `unsafe_code = "forbid"` |
| `clippy.toml` `disallowed-methods` | interdit les raccourcis qui contournent une règle | interdire `rand::thread_rng`, `Instant::now` dans le core (déterminisme) |
| Profils cargo | `release` LTO thin + `codegen-units=1`, profil `fast` pour itérer, `debug = "line-tables-only"` | repris tel quel |
| `mimalloc` en `#[global_allocator]` | allocateur rapide | optionnel, seulement si un benchmark le justifie |
| `horizon-anticheat` | vérifie côté serveur ce que le client envoie (timer, combat), niveaux de violation | validation d'un score par **rejeu déterministe** des inputs |
| `horizon-bench` / `horizon-perf` | benchmarks et traces de perf | `criterion` sur le tick du core |
| `xtask` | tâches de dev en Rust (`cargo xtask …`) | `cargo xtask ci` = fmt + clippy + test + deny |
| CI GitHub (`ci.yml`) | fmt, clippy `-D warnings`, tests `--locked` | idem + `cargo deny` + `cargo audit` |
| `AGENTS.md` / `CLAUDE.md` | carte du projet pour les agents | un `CLAUDE.md` court |

Ce qu'on **ne** reprend pas : système de mods WASM, télémétrie, launcher, patchs `third_party/`,
traductions, HDR. Tout ça est disproportionné pour un Flappy Bird.

## Structure proposée

```
flappy-bird/
├── Cargo.toml              # [workspace], lints et profils partagés
├── clippy.toml             # disallowed-methods (déterminisme du core)
├── deny.toml               # licences, advisories, crates bannies
├── rust-toolchain.toml     # toolchain épinglée
├── CLAUDE.md               # carte du projet
├── docs/layers.md          # la règle des dépendances
├── crates/
│   ├── flappy-core/        # PUR : physique, tuyaux, collisions, score, état. Zéro I/O, zéro graphique.
│   │                       #   tick fixe (ex. 60 Hz), RNG à graine, entrée = Vec<Action>
│   ├── flappy-replay/      # enregistre (graine + inputs par tick), rejoue, valide un score (« anticheat »)
│   ├── flappy-storage/     # high score / réglages : fichiers versionnés, validés, écriture atomique
│   ├── flappy-client/      # binaire : fenêtre, rendu, son, clavier → Action ; boucle à pas fixe
│   └── flappy-bench/       # criterion sur flappy-core
└── xtask/                  # cargo xtask ci | bench | layers
```

Règle (vérifiée par `cargo xtask layers` en CI, comme `check_layers.py`) :

| Crate | Peut dépendre de |
|---|---|
| flappy-core | rien du workspace (serde optionnel) |
| flappy-replay, flappy-storage | flappy-core |
| flappy-client | tout |
| flappy-bench, xtask | tout (outils) |

## Sécurité

1. `#![forbid(unsafe_code)]` dans toutes les crates (via `[workspace.lints.rust]`).
2. `cargo deny` : licences autorisées, advisories RustSec, doublons, sources git interdites.
3. `Cargo.lock` commité + builds `--locked` en CI.
4. Fichiers de sauvegarde = **frontière de confiance** : désérialisés avec `serde`, version de format,
   bornes vérifiées (score ≤ max plausible), écriture atomique (fichier temporaire + rename), jamais de
   `unwrap` sur une donnée externe.
5. Anti-triche du score (inspiré de `horizon-anticheat`) : un score n'est accepté que si le rejeu
   déterministe des inputs enregistrés redonne le même score. Utile si on ajoute un classement en ligne
   plus tard ; en local ça protège surtout contre l'édition manuelle du fichier.
6. Lints anti-panique en prod : `clippy::unwrap_used`, `expect_used`, `panic`, `indexing_slicing` en
   `deny` hors tests.

## Optimisation

1. **Pas fixe** (fixed timestep + accumulateur) : physique indépendante du FPS, rendu interpolé.
2. **Zéro allocation dans la boucle** : tuyaux dans un buffer circulaire de taille fixe (`ArrayVec` ou
   tableau), pas de `Vec` créé par frame.
3. Profils : `release` (LTO thin, `codegen-units = 1`, `panic = "abort"`), `fast` pour itérer,
   `dev` avec `opt-level = 1` pour les dépendances graphiques (sinon le jeu rame en debug).
4. Benchmarks `criterion` sur `World::tick` ; on ne fait aucune optimisation sans mesure avant/après.

## Qualité / workflow

- TDD dans `flappy-core` d'abord (tout est pur, donc testable sans fenêtre) : gravité, saut, collision,
  passage d'un tuyau = +1, game over, déterminisme (même graine + mêmes inputs = même état).
- `proptest` pour les invariants (l'oiseau ne traverse jamais un tuyau, le score ne décroît jamais).
- Mutation testing avec `cargo-mutants`.
- CI : `fmt --check`, `clippy --all-targets -D warnings`, `test --locked`, `deny check`, `xtask layers`.

## Ordre de livraison (tranches verticales)

1. **Squelette** : workspace, lints, profils, `deny.toml`, toolchain, CI, xtask. `cargo xtask ci` passe.
2. **Core jouable en headless** (TDD) : oiseau + gravité + saut + sol → game over.
3. **Tuyaux + score** (TDD) dans le core.
4. **Client minimal** : fenêtre, rectangles, clavier, boucle à pas fixe. Premier jeu jouable.
5. **Replay + validation de score**.
6. **Stockage sécurisé du high score**.
7. **Bench + polish** : sprites, son, interpolation, écran titre.

Chaque étape = un commit validé par toi avant de passer à la suivante.

## Décisions (2026-11-17)

- **Bibliothèque graphique** : `macroquad`.
- **Classement en ligne** : non. L'anti-triche protège le fichier local de high score.
- **Cible WASM/navigateur** : oui, dès le départ. Conséquences : le core n'utilise ni horloge, ni
  threads (bannis par `clippy.toml`) ; `gl.js` est copié depuis le crate `miniquad` verrouillé (pas
  de CDN) ; CSP stricte dans `web/index.html` ; le stockage passera par le `localStorage` du
  navigateur sur WASM.

## Avancement

- [x] 1. Squelette
- [x] 2. Core headless : oiseau, gravité, saut, plafond, sol
- [x] 3. Tuyaux + score (parcours dérivé de la graine, rien n'est stocké)
- [x] 4. Client minimal jouable (desktop + navigateur)
- [ ] 5. Replay + validation de score

## Risques acceptés

| Avis | Crate | Pourquoi on l'accepte | Quand revoir |
|---|---|---|---|
| RUSTSEC-2025-0035 (unsound) | macroquad 0.4.16 | Client mono-thread et hors ligne. Seul `flappy-client` voit macroquad, donc le remplacer (par miniquad seul ou wgpu) ne touche pas aux règles du jeu. | Si une version corrigée sort, ou si on ajoute du réseau ou des données non fiables qui passent par macroquad. |
| RUSTSEC-2026-0192 (unmaintained) | ttf-parser (via fontdue) | Il ne lit que des polices qu'on fournit nous-mêmes, jamais de police venant de l'extérieur. | Si macroquad change de moteur de polices. |

Les deux sont ignorés dans `deny.toml` par leur identifiant. Tout autre avis fait échouer la CI.
