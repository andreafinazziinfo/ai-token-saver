# RTK Complete Plan — Giugno 2026

## Executive summary

RTK ha già una base solida come motore Rust per filtrare output CLI, comprimere contesto, archiviare memoria locale con SQLite FTS5 e imporre disciplina sull'output dell'agente.[cite:2] Lo stato dell'arte del 2026 però si è spostato da semplici “token savers” a **context engines** che combinano filtering, indexing strutturale del codebase, memoria multi-livello, session handoff, MCP tool surface minima e osservabilità dei costi.[cite:2][cite:66][cite:80][cite:82][cite:90]

Il piano completo per RTK deve quindi unire quattro assi: **engine tecnico**, **context engineering**, **finops/benchmarking** e **marketing/prova pubblica**. L'obiettivo non è solo “risparmiare token”, ma diventare un **Rust Context Engine for AI Coding Agents**: single binary, misurabile, integrabile con shell e MCP, credibile nei numeri, e facile da adottare da developer e piccoli team.[cite:2][cite:81][cite:84][cite:90][cite:119]

## Obiettivo del progetto

RTK deve evolvere da toolkit di compressione dei token a piattaforma Rust-first che ottimizza **input, middle, output, memoria, tool surface e session lifecycle** per gli AI coding agents.[cite:66][cite:80][cite:121] In pratica il prodotto deve:

- ridurre token e costi;
- ridurre latenza e rumore;
- mantenere qualità e correttezza del contesto;
- offrire metriche verificabili e benchmark credibili;
- diventare un reference project serio per context engineering su agenti di coding.[cite:60][cite:63][cite:121][cite:130]

## Stato attuale di RTK

La repo attuale presenta già questi pilastri:[cite:2]

- input virtualization su comandi rumorosi come git, cargo, pytest, docker, npm;
- caching e retrieval via SQLite FTS5;
- comando `rtk think` per hidden reasoning offload;
- `rtk memory` per memoria persistente di progetto;
- `rtk pack` per packaging del contesto;
- DLP redaction e guardrail su comandi pericolosi;
- output discipline tramite profili/rules in stile Caveman/Ponytail.

Questa architettura è coerente con la traiettoria moderna del context engineering, ma è ancora incompleta rispetto ai competitor più forti su tre aree: **code indexing strutturale**, **MCP server/tool surface minima**, e **finops/benchmark/reporting di livello prodotto**.[cite:82][cite:90][cite:103][cite:121]

## State of the art — giugno 2026

### 1. Più contesto non equivale a risultati migliori

Nel 2026 il consenso pratico è che il degrado non dipende solo dal costo della finestra di contesto, ma anche dalla qualità del materiale inserito: oltre una certa soglia, grandi blocchi di contesto peggiorano focus, retrieval effettivo e capacità dell'agente di usare l'informazione giusta al momento giusto.[cite:66][cite:74][cite:77][cite:80] Il problema chiave è il **context distraction/confusion**, non il semplice numero di token.[cite:74][cite:80]

### 2. Dalla prompt engineering alla context engineering

Il baricentro del settore si è spostato verso pipeline che combinano retrieval selettivo, context layering, compaction iterativa, memory tiering, prompt caching e tool surface minima.[cite:66][cite:72][cite:80][cite:121] I team migliori non “buttano dentro più file”, ma decidono cosa far entrare, in che ordine, con quale priorità e con quale policy di rinnovo.[cite:61][cite:66][cite:74]

### 3. I tool outputs sono il principale killer del budget

Log di test, build, linter, docker, diff e file dump rappresentano una quota enorme del consumo inutile nei coding agents, spesso superiore al testo naturale della conversazione stessa.[cite:60][cite:63][cite:121][cite:128] Da qui l'esplosione di strumenti che lavorano su filtering, compaction, indexing e tool wrapping come LeanCTX, Code Context Engine, Token Savior e tool analoghi.[cite:81][cite:82][cite:93][cite:103]

### 4. Multi-tier memory e session discipline sono ormai standard impliciti

I sistemi migliori usano almeno tre livelli di memoria: hot memory per il task corrente, warm memory per lo stato della sessione e cold memory per decisioni e fatti persistenti di progetto.[cite:74][cite:80] Inoltre, la compaction viene attivata molto prima della saturazione della finestra di contesto, spesso tra 40% e 70%, e non quando è già troppo tardi.[cite:77][cite:80]

### 5. Pattern dominante: context as IDE

Nel coding moderno il contesto non è più una semplice chat lunga, ma un ambiente strutturato: AGENTS.md/CLAUDE.md curati, project memory file, naming coerente, code graph, simboli e dipendenze interrogabili.[cite:61][cite:68][cite:74][cite:78] RTK è vicino a questo paradigma, ma gli manca ancora il layer **index/graph + MCP exposure** che oggi distingue i tool più completi.[cite:2][cite:82][cite:90][cite:103]

### 6. Convergenza degli stack

Si osserva una convergenza abbastanza netta: motori **Rust single-binary** per filtering/compressione e server **MCP** per tool di indexing/search/memory, con crescente uso di routing verso modelli più piccoli o locali per le fasi meno critiche.[cite:73][cite:81][cite:84][cite:90][cite:103] Questa convergenza rafforza la scelta di RTK di restare Rust-first, ma suggerisce di aggiungere un surface MCP minima invece di rimanere solo nella dimensione shell wrapper.[cite:2][cite:90]

## Principi guida del piano

L'intero piano deve seguire questi principi:

1. **Rust-first**: il kernel resta in Rust; JS/TS, se presenti, sono solo edge adapters.[cite:81][cite:84][cite:90]
2. **Single source of truth**: pricing, benchmark, cost calculator, docs generator e audit devono leggere dagli stessi dati.[cite:108][cite:118][cite:119]
3. **Determinismo e receipts**: ogni claim di risparmio deve essere difendibile con metadata, fixture, tokenizer, commit SHA e pricing revision.[cite:107][cite:109][cite:111]
4. **Minimal tool surface**: pochi tool ben progettati battono decine di tool ridondanti.[cite:14][cite:66][cite:72]
5. **Token savings come conseguenza, non unica identità**: il positioning deve evolvere verso “context engine” o “agent runtime optimizer”.[cite:66][cite:119]

## Copertura completa dei campi

Il piano definitivo copre questi domini.

### A. Product & positioning

- category design del prodotto;
- naming e messaggio principale;
- segmentazione audience;
- proposizione di valore per solo dev, power user, OSS maintainer, small team;
- differentiatori rispetto ai competitor.

### B. Engine tecnico

- shell hook e wrapper CLI;
- filters content-aware;
- DLP e safety guardrails;
- pack del contesto;
- `rtk think`;
- `rtk memory`;
- artifact manager;
- indexing AST/symbol/graph;
- MCP server Rust-first;
- policy engine;
- plugin/filter modules.

### C. Context engineering

- input filtering;
- retrieval ibrido;
- multi-tier memory;
- anchored summaries;
- handoff/compact/reset;
- tool sparsity;
- tool description minimizzate;
- session-state document.

### D. FinOps e data layer

- pricing registry;
- pricing revisioning;
- cost calculator condiviso;
- benchmark exports JSON/CSV;
- report audit;
- model routing helpers;
- budget caps e alerts.

### E. Docs, credibility e proof assets

- README auto-aggiornato;
- docs/pricing.md;
- docs/benchmarks.md;
- docs/use-cases.md;
- docs/limitations.md;
- changelog;
- fixtures ufficiali;
- release content generator;
- public proof assets e case studies.

### F. Governance e adoption

- doctor commands;
- install scripts per client;
- templates AGENTS/CLAUDE;
- upgrade/migration notes;
- observability exports;
- release process.

## Best practices operative per RTK

### INPUT

#### 1. Source-level filtering

RTK deve continuare a filtrare i dati alla sorgente, cioè prima che shell output e file grezzi entrino nel prompt.[cite:2][cite:81][cite:93][cite:99] Questo resta il livello con il miglior rapporto impatto/complessità, perché gran parte dei token spesi dagli agenti di coding è puro rumore operativo.[cite:60][cite:63][cite:121]

**Da fare in RTK**

- espandere la matrice di filtri per più tool e linguaggi;
- introdurre modalità diverse per tipo di output: error-focused, summary-first, schema-extract, AST-skeleton;
- aggiungere test golden su output grezzo vs filtrato.

#### 2. AST indexing e symbol retrieval

Le repo grandi non devono essere lette come testo lineare ma interrogate per simbolo, dipendenze, referenze e impatto dei cambiamenti.[cite:54][cite:82][cite:91][cite:103] È una delle evoluzioni più importanti per RTK se vuole competere con strumenti come Code Context Engine o Token Savior.[cite:82][cite:103]

**Da fare in RTK**

- crate `rtk-index` basata su Tree-sitter;
- DB schema per simboli, file, dipendenze, call edges;
- comandi: `rtk symbols`, `rtk refs`, `rtk deps`, `rtk impact`, `rtk search`.

#### 3. Retrieval ibrido

BM25/FTS5 offre ottima base lessicale, ma il 2026 si sta muovendo verso retrieval ibrido BM25 + embeddings per recall più robusto.[cite:66][cite:78][cite:80] RTK dovrebbe adottare una strategia modulare: BM25/FTS5 come default, embeddings come layer opzionale e locale.[cite:66][cite:78]

**Da fare in RTK**

- mantenere SQLite FTS5 come baseline;
- aggiungere modulo ONNX embeddings opzionale in Rust;
- combinare keyword score + embedding score + metadata score.

### MIDDLE

#### 4. Multi-tier memory

Le sessioni lunghe richiedono un layering esplicito della memoria: hot, warm, cold.[cite:74][cite:80] RTK ha già `rtk memory`, ma deve formalizzarne il ruolo e costruirci sopra strumenti più opinionati.[cite:2]

**Da fare in RTK**

- hot memory = ultimi task e fatti attivi;
- warm memory = session-state document;
- cold memory = chiavi persistenti di progetto, decisioni e artefatti.

#### 5. Anchored iterative summarization

Le sessioni non vanno ricompattate “da zero” ogni volta, ma mantenute tramite un documento di stato aggiornato iterativamente.[cite:80][cite:64] Questo riduce drift e costi, e rende i reset di sessione molto più robusti.[cite:80]

**Da fare in RTK**

- `rtk session-state init|get|update|export`;
- formato JSON/YAML leggibile;
- hook/prompt pronti per reiniettare lo state nei client.

#### 6. Hidden reasoning offload

`rtk think` è già una feature perfettamente allineata con le pratiche più moderne di “reasoning fuori dal contesto utente”.[cite:2][cite:80] Va però resa più first-class e più integrata nell'ecosistema docs/client.

**Da fare in RTK**

- snippet ufficiali per Claude Code, Cursor, Gemini CLI;
- `rtk think inspect` e `rtk think gc`;
- ergonomia migliore nella documentazione.

### OUTPUT

#### 7. Output profiles

La compressione dell'output non deve dipendere solo da regole statiche, ma da profili riutilizzabili a livello di team o progetto.[cite:2][cite:83][cite:94] Questo aiuta sia il risparmio token sia la qualità percepita del tool.

**Da fare in RTK**

- `output_profiles` in `.rtk.json`;
- preset `strict`, `balanced`, `developer`, `audit`, `json-only`;
- lint dei profili per evitare stili contraddittori.

#### 8. Artifact spillover

Per output lunghi il pattern vincente è salvare tutto come artifact e far passare nel contesto solo preview + reference.[cite:32][cite:80] RTK ha già un embrione di questa idea con `rtk show-log`, ma va generalizzata.[cite:2]

**Da fare in RTK**

- `rtk artifact list|get|open|gc`;
- tipi: cli-log, reasoning, summary, pack snapshot, report;
- integrazione con eventuale MCP server.

### MEMORY

#### 9. AGENTS.md / CLAUDE.md come interfacce di memoria

Questi file sono ormai parte integrante del sistema di context engineering e non semplice documentazione accessoria.[cite:61][cite:68][cite:74] RTK deve trasformarli in oggetti mantenuti e verificabili.

**Da fare in RTK**

- `rtk agents init`;
- `rtk agents doctor`;
- `rtk agents compact`;
- template diversi per solo-dev, team, OSS, mono-repo.

#### 10. Memory discipline & overwrite

Le memorie vecchie o incoerenti sono uno dei modi più frequenti con cui i coding agents si “avvelenano”.[cite:74][cite:80] Per questo serve una disciplina esplicita di overwrite e verifica.

**Da fare in RTK**

- `rtk memory overwrite`;
- `rtk memory doctor`;
- validazione di chiavi duplicate, stale, contraddittorie.

### TOOL SURFACE

#### 11. Tool sparsity e routing

Troppi tool peggiorano sia selezione sia consumo di contesto.[cite:14][cite:66][cite:72] RTK deve offrire una surface MCP minima ma forte.

**Tool MCP core consigliati**

- `search_code`
- `find_symbols`
- `find_refs`
- `project_memory`
- `artifact_get`
- `context_pack`
- `session_state`

#### 12. Tool descriptions compatte

Le descrizioni dei tool e i loro JSON schema devono essere progettati come budget token, non come documentazione prolissa.[cite:63][cite:66][cite:72]

**Da fare in RTK**

- generatore tool schema compatto;
- esempi minimi;
- naming coerente e corto.

### SESSIONS

#### 13. Compaction a 40–70%

La compaction tardiva è una strategia perdente; bisogna attivarla in anticipo e con stato ancorato.[cite:77][cite:80] RTK può fornire pattern, script e prompt anche se non controlla direttamente il contesto interno del client.

**Da fare in RTK**

- comando/helper `/rtk compact`;
- guidelines ufficiali per i client;
- template di session policy.

#### 14. Episodic workflow

Le sessioni lunghe dovrebbero essere spezzate in episodi con stato importato, non tenute vive all'infinito.[cite:63][cite:80] Questo pattern è altamente compatibile con `rtk session-state` e `rtk artifact`.

## Competitor e feature strategy

### Cosa prendere dai competitor come idee


| Competitor          | Idee/feature da prendere                                                                                               | Da NON copiare                                                  | Note licenza                                                                                            |
| ------------------- | ---------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| LeanCTX             | dual shell+MCP, project graph, signed savings ledger, positioning “context layer” [cite:81][cite:84][cite:90][cite:93] | naming, branding, copy, eventuale codice senza verifica LICENSE | licenza da verificare [cite:81]                                                                         |
| Token Savior        | symbol navigation, dependency graph, impact analysis, surgical retrieval [cite:54][cite:103]                           | naming troppo vicino, docs, benchmark claims                    | MIT, quindi riuso codice possibile con attribuzione; meglio reimplementare in Rust [cite:100][cite:103] |
| Code Context Engine | indexing + retrieval + benchmark reproducibility [cite:82][cite:85][cite:101]                                          | copy di benchmark framing o docs                                | MIT nelle fonti disponibili [cite:95][cite:101]                                                         |
| Context Mode        | multi-platform adapters, session continuity patterns [cite:20][cite:46]                                                | codice/docs perché ELv2 non è permissiva come MIT/Apache        | Elastic License 2.0 [cite:34]                                                                           |
| ToolRAG             | tool surface minima, tool selection dinamica [cite:14]                                                                 | implementazione specifica, naming                               | Apache-2.0 [cite:14]                                                                                    |


## Piano definitivo completo

## Sezione 1 — Architettura target

### Visione architetturale

RTK deve convergere verso un'architettura a 4 crate:

- `rtk-core`: parsing CLI, filtri, DLP, policies, wrappers;
- `rtk-db`: SQLite/FTS5, artifacts, stats, pricing registry, session state;
- `rtk-index`: Tree-sitter, symbols, deps, graph, optional embeddings;
- `rtk-mcp`: server MCP, tool surface, install helpers.

### Configurazione target

`.rtk.json` o `.rtk.toml` deve diventare la source of truth locale per:

- output profiles;
- compaction policy;
- DLP rules;
- denylist/allowlist comandi;
- indexing scope;
- model routing hints;
- budget e alert;
- audit/report settings.

## Sezione 2 — Piano tecnico completo

### P0 — 0/30 giorni

#### Engine

- hardening filtri esistenti;
- filtri content-aware;
- `rtk stats` v1;
- `rtk doctor` v1;
- `rtk agents init/doctor`;
- `rtk session-state` v1.

#### Data/FinOps

- `pricing/models.toml` o `data/model_pricing.json`;
- `pricing_revision` e `source_type`;
- modulo cost calculator condiviso;
- `output/benchmarks.json` e `output/benchmarks.csv`;
- metadata minimi: timestamp, tokenizer, commit_sha, benchmark_version.

#### Docs/Marketing

- `docs/pricing.md`;
- micro-sezione README “How we calculate savings”;
- primi statement metodologici;
- claim ripuliti e più rigorosi.

### P1 — 30/60 giorni

#### Engine

- `rtk artifact`;
- prototipo `rtk index` su Rust e TS;
- `rtk symbols`, `rtk deps`, `rtk search`, `rtk refs`;
- design API MCP.

#### Data/FinOps

- `rtk audit` MVP;
- `fixtures/` ufficiali;
- use-case benchmark oltre ai semplici comandi CLI: single-file edit, bug investigation, PR review, refactor.

#### Docs/Marketing

- README auto-aggiornato;
- `docs/benchmarks.md`;
- `docs/use-cases.md`;
- `docs/limitations.md`;
- `CHANGELOG.md` strutturato.

### P2 — 60/90 giorni

#### Engine

- `rtk mcp serve`;
- surface MCP minimale;
- hook/install one-liner per Claude Code, Cursor, Gemini CLI;
- policy base per compaction e session handoff;
- model routing helpers.

#### Data/FinOps

- audit report markdown + json;
- budget caps e alert;
- pricing checker;
- richer trust metadata: OS, arch, shell, pricing revision, cost model version.

#### Docs/Marketing

- release content generator;
- benchmark summary generator;
- blog post “From Token Saver to Rust Context Engine”;
- snippet per social/release.

### P3 — oltre 90 giorni

#### Engine

- project graph completo;
- semantic memory integrata al graph;
- optional embeddings locali;
- compression policy engine avanzato;
- observability export per dashboard.

#### Marketing / GTM

- landing page dedicata ai costi;
- case study templates;
- pagina “Why RTK” per confronto di categoria;
- rebranding progressivo verso “Rust Context Engine”.

## Sezione 3 — Piano marketing completo

### Narrative principale

Il messaggio principale non dovrebbe essere soltanto “riduce i token”, ma:

> RTK è il contesto giusto, nel formato giusto, al momento giusto — per coding agents che costano meno e sbagliano meno.

### Pillars di messaging

1. **Rust-first performance** — single binary, basso overhead, portabilità.[cite:81][cite:84][cite:90]
2. **Measured savings** — benchmark, pricing registry, audit, fixtures.[cite:107][cite:109][cite:111]
3. **Context correctness** — indexing, memory, compaction, state discipline.[cite:66][cite:74][cite:82]
4. **Safety & control** — DLP, deny rules, command guardrails.[cite:2]
5. **Adoption simplicity** — install facile, one-liner hooks, docs serie.[cite:2][cite:90]

### Audience principali


| Audience                 | Pain                                             | Promessa RTK                               |
| ------------------------ | ------------------------------------------------ | ------------------------------------------ |
| Solo developer           | costi variabili, chat degradate, log rumorosi    | meno rumore, meno costo, più focus         |
| Power user Claude/Cursor | context rot, sessioni lunghe, output dispersivi  | session discipline + memory + filtering    |
| OSS maintainer           | repo grandi, benchmark credibili, demo pubbliche | fixtures, audit, docs e numeri difendibili |
| Small engineering team   | costi agent fuori controllo, poca observability  | audit, pricing, benchmark, governance lite |


### Asset marketing necessari

- README riscritto con positioning moderno;
- docs/pricing.md;
- docs/benchmarks.md;
- docs/use-cases.md;
- docs/limitations.md;
- changelog pulito;
- benchmark screenshots;
- examples/fixtures;
- release notes generator;
- case study template;
- future landing page.

## Sezione 4 — Piano Antigravity completo

Antigravity può essere il maintainer automatico del layer dati/docs/marketing.

### Responsabilità Antigravity

- mantenere pricing registry;
- lanciare benchmark e generare export;
- calcolare savings con il cost calculator condiviso;
- aggiornare README/docs;
- generare release notes e benchmark summaries;
- creare/aggiornare fixtures demo;
- eseguire audit periodici e salvarne i report.

### Deliverable finali attesi da Antigravity

- `pricing/models.toml` oppure `data/model_pricing.json`;
- cost calculator module;
- benchmark export JSON/CSV;
- `rtk audit` MVP;
- README aggiornato;
- docs base complete;
- fixtures demo;
- changelog;
- release content generator.

## Sezione 5 — Budget, metriche e KPI

### KPI di prodotto

- median saving % per comando;
- p95 token reduction per scenario;
- numero di tool supportati;
- tempo medio risparmiato per comando;
- ratio raw/filtered;
- adoption di `rtk memory`, `rtk think`, `rtk pack`, `rtk audit`.

### KPI di credibility

- percentuale benchmark con fixture pubblica;
- benchmark reproducibility score;
- copertura metadata di fiducia;
- pricing freshness (giorni dall'ultima verifica).

### KPI di marketing

- star growth;
- docs page views;
- benchmark page visits;
- conversione install → first benchmark;
- numero di case studies o testimonial tecnici.

## Sezione 6 — Cosa copre il piano e cosa resta fuori

### Coperto dal piano

- product positioning;
- architecture target;
- engine roadmap;
- context engineering best practices;
- pricing/finops layer;
- benchmark infrastructure;
- docs e credibility assets;
- marketing narrative;
- adoption e governance lite;
- deliverable concreti e milestone.

### Fuori dal piano per ora

- enterprise SaaS hosted;
- dashboard web full-fledged;
- team billing remoto;
- cloud sync multi-utente;
- managed embeddings service;
- agent orchestration completa cross-model in produzione.

Queste aree possono restare escluse finché RTK non consolida la propria identità come motore locale Rust-first.[cite:81][cite:84][cite:119]

## Sezione 7 — Raccomandazione finale

La direzione più forte per RTK è trasformarsi da “token saver” a **Rust Context Engine** con quattro promesse semplici: meno rumore, meno costo, più controllo, contesto più corretto.[cite:66][cite:81][cite:84][cite:119] Il vantaggio competitivo naturale non è diventare un generico MCP server, ma un **motore locale deterministico** che unisce shell hooks, memory, indexing, audit e proof assets in un pacchetto unico e misurabile.[cite:2][cite:90][cite:93]

In pratica, la priorità strategica è questa:

1. consolidare il layer dati/benchmark/pricing/audit;
2. aggiungere `rtk index` e `rtk artifact`;
3. esporre una MCP surface minima;
4. riposizionare pubblicamente RTK come context engine;
5. usare Antigravity per automatizzare docs, benchmark e release content.

Questo è il punto in cui RTK smette di essere solo un buon tool di compressione e diventa una reference architecture credibile per AI coding agents Rust-first nel 2026.[cite:82][cite:90][cite:121]











## **Aggiunta 1.1: Code graph in Rust (P1)**

Nel piano tecnico, P1.1 diventa:

**P1.1 — Code graph ibrido (AST + semantic)**

- crate `rtk-index` con Tree-sitter + petgraph
- modello: simbolo → file → dependency → impacto
- comandi: `rtk symbols`, `rtk deps`, `rtk graph`, `rtk impact`
- non usare librerie AI tools, ma sole Rust libs

## **Aggiunta 1.2: Integration point per Obsidian (P2)**

Nel piano engineering, aggiungi P2.4:

**P2.4 — Integration point per Obsidian (opzionale)**

- non integrare Obsidian nel core
- invece: esporta `rtk graph` in formato compatibile con Obsidian graph
- plugin Obsidian separato che legge export RTK
- così rimieni Rust-first senza dipendere da Obsidian

## **Aggiunta 1.3: Audit per knowledge graph (P1)**

Nel piano data/finops, aggiungi:

**P1.5 — Audit per graph & indexes**

- `rtk audit graph` mostra:
  - simboli indicizzati
  - dependency edges
  - query performance
  - graph coverage
- metriche: symbols count, edges count, query latency


## Sezione 8 — Proof of Work & Validation

### 1. Principio Chiave
La roadmap **NON** è un documento da scrivere e dimenticare. È un documento vivo che:
- Costringe a fare le cose in ordine sequenziale e logico.
- Traccia in tempo reale cosa è stato completato e validato.
- Valida che il progetto e le sue evoluzioni siano reali e funzionanti.
- Comunica a investitori, utenti e al team che lo sviluppo segue criteri di engineering seri e misurabili.

### 2. Strategia di Proof of Work per Fase

#### Fase 1: P0 = Foundation (30 giorni)
**Output concreti:**
- `data/model_pricing.json` aggiornato
- `pricing.rs` funzionante
- benchmark export JSON/CSV
- `rtk doctor` che passa
- `rtk session-state` che salva
- `rtk agents` che genera file
- `rtk think inspect/gc` funzionanti

**Validazione:**
```bash
# Test automatico
cargo test

# Build release
cargo build --release

# Verifica binario
ls -la target/release/rtk  # Deve essere < 5MB

# Verifica manuale
rtk doctor
rtk benchmark export --format json --output bench.json
rtk session-state init
rtk session-state get
rtk agents init --template solo-dev
rtk think inspect
```

#### Fase 2: P1 = Indexing + Docs (30-60 giorni)
**Output concreti:**
- `rtk-index` crate funzionante
- `rtk symbols/deps/refs/impact/search` che interrogano il DB dell'indice
- `docs/pricing.md` con metodologia
- `docs/benchmarks.md` con risultati
- `docs/use-cases.md` con scenari
- `docs/limitations.md` honest
- `fixtures/` con golden test data

**Validazione:**
```bash
# Test index
cargo test --package rtk-index

# Query demo
rtk symbols find --name "main"
rtk deps show --file "src/main.rs"
rtk refs find --symbol "calculate_cost"
rtk impact analyze --file "src/dlp.rs"

# Verify docs
mdbook serve docs/  # Deve buildare e visualizzarsi

# Verify fixtures
cargo test --test fixtures  # Golden tests devono passare
```

#### Fase 3: P2 = MCP + Budget (60-90 giorni)
**Output concreti:**
- `rtk-mcp` crate compilato
- MCP server che risponde a 7 tool
- `rtk mcp install --client claude` funzionante
- Budget caps che triggerano alert
- Model routing che suggerisce modelli
- `rtk memory doctor/overwrite` funzionanti

**Validazione:**
```bash
# Test MCP
cargo test --package rtk-mcp

# Install MCP
rtk mcp install --client claude

# Verify tools
rtk mcp call search_code --query "token"
rtk mcp call find_symbols --name "calculate"
rtk mcp call artifact_get --id "abc123"

# Verify budget
rtk budget check --limit 50.00
rtk model suggest --task "single-file-edit"
```

#### Fase 4: P3 = Advanced (90+ giorni)
**Output concreti:**
- Hybrid retrieval (BM25 + ONNX)
- Obsidian graph export .md
- `rtk audit graph` con metriche

**Validazione:**
```bash
# Test retrieval
cargo test --package rtk-index --features embeddings

# Export graph
rtk graph export --format obsidian --output obsidian/

# Verify audit
rtk audit graph
```

### 3. Timeline di Proof of Work

| Giorno | Milestone | Proof Artifact / Verifica |
|---|---|---|
| **Day 7** | Pricing registry loaded | `rtk doctor` → ✅ Pricing OK |
| **Day 14** | Benchmark export working | `bench.json` con metadata |
| **Day 21** | Session state working | JSON export con structure |
| **Day 30** | **P0 Completed** | `cargo test` ✅ + release build < 5MB |
| **Day 45** | Index working | Symbol query demo + deps graph |
| **Day 60** | **P1 Completed** | mdBook build + fixtures pass |
| **Day 75** | MCP working | Claude Code integration + tool call |
| **Day 90** | **P2 Completed** | Budget alert + model routing |
| **Day 120** | **P3 Completed** | Embeddings + Obsidian export |

### 4. Metriche di Validazione e Qualità

| Metrica | Target | Come misurare |
|---|---|---|
| **Tests pass rate** | 100% | `cargo test` |
| **Release build size** | < 5MB | `ls -la target/release/rtk` |
| **Docs build** | No errors | `mdbook build docs/` |
| **Fixtures pass** | 100% | `cargo test --test fixtures` |
| **Benchmark export** | JSON valido | `jq . bench.json` |
| **MCP tools** | 7 working | `rtk mcp call <tool>` |
| **Budget alert** | Trigger < limit | Simulate usage |


