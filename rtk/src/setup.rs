use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

const LAZY_DEV_CONTENT: &str = r#"---
description: Principi Senior Developer — YAGNI, codice minimo, modifiche mirate e diff corti per efficienza token.
alwaysApply: true
---

# Lazy Developer (YAGNI & Efficiency)

Adotta la mentalità del "miglior codice è quello che non è stato scritto" (YAGNI - You Aren't Gonna Need It) per massimizzare la chiarezza e ridurre il consumo di token.

## Regole Comportamentali

1. **Codice Minimo**: Scrivi solo le righe necessarie per implementare la feature o risolvere il bug. Evita di aggiungere astrazioni premature, classi helper o logica "per il futuro".
2. **Modifiche Mirate (Diff Corti)**: Quando modifichi file esistenti, mantieni il delta il più piccolo possibile.
   - Non riscrivere interi file o intere funzioni per modesti cambiamenti (es. ≤3 righe). Usa modifiche mirate.
   - Non formattare parti del file non correlate alla tua patch.
3. **Fiducia nella Standard Library**: Usa funzioni native del linguaggio e framework correnti. Non aggiungere nuove dipendenze o librerie esterne a meno che non sia esplicitamente richiesto o bloccante.
4. **Nessun Recap o Narrazione**: Procedi direttamente alle modifiche. Non riassumere i tuoi compiti né ripetere le istruzioni dell'utente.

## Ladder of Laziness
- Livello 1: Risolvi il problema con il minor numero di righe modificate.
- Livello 2: Usa le funzioni già esistenti nel codebase.
- Livello 3: Evita modifiche strutturali se una semplice correzione locale è sufficiente.
"#;

const TOKEN_EFFICIENCY_CONTENT: &str = r#"---
description: Token caps misurabili — BRIEF default, batch letture, no policy spam per efficienza token.
alwaysApply: true
---

# Token efficiency

## Modalità

- **BRIEF** — default operativo (patch, bugfix, routine).
- **STANDARD** — solo con **superfici multiple** o spiegazione necessaria (non audit formale).
- **AUDIT** — solo audit / security / architecture review **dichiarati**.

## Caps (operativi)

- Prima riga di ogni risposta: `Mode: BRIEF|STANDARD|AUDIT | Repo: <attivo>`.
- Max **3** file letti per batch; stop e rivaluta salvo **AUDIT** esplicito.
- Summary finale post-patch: **≤ 8 righe** (file toccati + comando test o checklist).
- **GRAPH_REPORT** / grafi: **vietato** wall of text; estratti brevi (max 10 righe).
- **Vietata** la duplicazione in chat di lunghi tratti di policy o regole.

## Skill

- **Minimizzare** stacking; una primaria + secondaria solo se a valore verificabile.
- **Non caricare** skill lunghe senza trigger chiaro.
- Planning (sparc, sequential, hermes): **no** per patch banali / one-liner.
- **codeburn**: per audit / perf / review pesante — non per ogni fix.
- **context-mode** (`skills/context-mode-routing.md`): raccomandato per molte letture.
- **caveman/brief**: (`skills/caveman/SKILL.md`).

## MCP

- I server MCP (come GitNexus / Graphify o altri indicizzatori) possono be **stale** rispetto allo stato corrente di HEAD.
- Non fare affidamento solo sulle risposte del grafo se una ricerca locale (`grep` / `git diff`) può confermare lo stato dei file.
"#;

pub fn run_init() -> Result<()> {
    println!("⚙️ Bootstrapping AI Efficiency rules in the current directory...");

    let cursor_rules_dir = Path::new(".cursor/rules");
    let agents_rules_dir = Path::new(".agents/rules");

    // Create directories
    fs::create_dir_all(cursor_rules_dir)
        .context("failed to create .cursor/rules directory")?;
    fs::create_dir_all(agents_rules_dir)
        .context("failed to create .agents/rules directory")?;

    // Write rule files
    fs::write(cursor_rules_dir.join("lazy-dev.mdc"), LAZY_DEV_CONTENT)
        .context("failed to write lazy-dev.mdc to .cursor/rules")?;
    fs::write(cursor_rules_dir.join("token-efficiency.mdc"), TOKEN_EFFICIENCY_CONTENT)
        .context("failed to write token-efficiency.mdc to .cursor/rules")?;

    fs::write(agents_rules_dir.join("lazy-dev.mdc"), LAZY_DEV_CONTENT)
        .context("failed to write lazy-dev.mdc to .agents/rules")?;
    fs::write(agents_rules_dir.join("token-efficiency.mdc"), TOKEN_EFFICIENCY_CONTENT)
        .context("failed to write token-efficiency.mdc to .agents/rules")?;

    println!("✅ Created rules inside .cursor/rules/ and .agents/rules/");
    println!("");
    println!("==========================================================");
    println!("🎉 RTK AI Rules Bootstrapped Successfully!");
    println!("==========================================================");
    println!("To complete your setup:");
    println!("1. Activate transparent terminal filtering in Claude Code by adding");
    println!("   the PreToolUse hook to your .claude/settings.json:");
    println!("");
    println!("   \"hooks\": {{");
    println!("     \"PreToolUse\": [");
    println!("       {{");
    println!("         \"matcher\": \"Bash\",");
    println!("         \"hooks\": [");
    println!("           {{");
    println!("             \"type\": \"command\",");
    println!("             \"command\": \"bash /path/to/ai-efficiency-toolkit/hooks/rtk-rewrite.sh\",");
    println!("             \"timeout\": 5000");
    println!("           }}");
    println!("         ]]");
    println!("       }}");
    println!("     ]");
    println!("   }}");
    println!("");
    println!("2. Add shell aliases to your ~/.bashrc or ~/.zshrc for CLI wrappers:");
    println!("   alias git=\"rtk git\"");
    println!("   alias cargo=\"rtk cargo\"");
    println!("   alias pytest=\"rtk pytest\"");
    println!("   alias ls=\"rtk ls\"");
    println!("   alias npm=\"rtk npm\"");
    println!("==========================================================");

    Ok(())
}
