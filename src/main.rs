use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use console::{pad_str, Alignment};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

mod translations;
use translations::{translate, ui, Lang};

const REPO: &str = "eyjvw/BetterNorminette";

#[derive(Parser, Debug)]
#[command(name = "better-norminette",
          disable_help_flag = true,
          disable_help_subcommand = true,
          disable_version_flag = true)]
struct Cli {
    /// files or directories to check
    paths: Vec<PathBuf>,

    #[arg(short, long)]
    lang: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// persist the display language (en, fr, es)
    #[command(disable_help_flag = true)]
    Lang { lang: String },
    #[command(disable_help_flag = true)]
    Update,
    #[command(disable_help_flag = true)]
    Uninstall,
    #[command(disable_help_flag = true)]
    Version,
    #[command(disable_help_flag = true)]
    Help,
}

fn config_path() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(PathBuf::from(home).join(".betternorminette").join("lang"))
}

fn resolve_lang(flag: &Option<String>) -> Lang {
    if let Some(l) = flag.as_deref().and_then(Lang::parse) {
        return l;
    }
    if let Some(l) = std::env::var("BETTERNORMINETTE_LANG").ok().as_deref().and_then(Lang::parse) {
        return l;
    }
    if let Some(l) = config_path()
        .and_then(|p| fs::read_to_string(p).ok())
        .as_deref()
        .map(str::trim)
        .and_then(Lang::parse)
    {
        return l;
    }
    // fall back to the system locale
    if let Some(l) = std::env::var("LANG").ok().and_then(|v| Lang::parse(&v[..2.min(v.len())])) {
        return l;
    }
    Lang::En
}

fn save_lang(lang: Lang) -> Result<()> {
    let p = config_path().ok_or_else(|| anyhow::anyhow!("no HOME"))?;
    fs::create_dir_all(p.parent().unwrap())?;
    fs::write(&p, lang.code())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// norminette output parsing
// ---------------------------------------------------------------------------

struct Issue {
    code: String,
    line: u32,
    col: u32,
    original: String,
    is_notice: bool,
}

struct FileReport {
    path: String,
    ok: bool,
    issues: Vec<Issue>,
}

/// Parse `Error: CODE (line: N, col: M): message` / `Notice: ...` lines.
fn parse_issue(line: &str, is_notice: bool) -> Option<Issue> {
    let rest = line.split_once(": ")?.1; // drop "Error"/"Notice"
    let (code, rest) = rest.split_once('(')?;
    let (pos, msg) = rest.split_once("):")?;
    let mut ln = 0u32;
    let mut col = 0u32;
    for part in pos.split(',') {
        let (k, v) = part.split_once(':')?;
        let v = v.trim().trim_end_matches(')');
        match k.trim() {
            "line" => ln = v.parse().unwrap_or(0),
            "col" => col = v.parse().unwrap_or(0),
            _ => {}
        }
    }
    Some(Issue {
        code: code.trim().to_string(),
        line: ln,
        col,
        original: msg.trim().to_string(),
        is_notice,
    })
}

fn run_norminette(paths: &[PathBuf]) -> Result<Vec<FileReport>> {
    let out = Command::new("norminette")
        .args(paths)
        .output()?;
    let stdout = String::from_utf8_lossy(&out.stdout);

    let mut reports: Vec<FileReport> = Vec::new();
    for line in stdout.lines() {
        if let Some(path) = line.strip_suffix(": OK!") {
            reports.push(FileReport { path: path.to_string(), ok: true, issues: vec![] });
        } else if let Some(path) = line.strip_suffix(": Error!") {
            reports.push(FileReport { path: path.to_string(), ok: false, issues: vec![] });
        } else if line.starts_with("Error: ") {
            if let (Some(issue), Some(last)) = (parse_issue(line, false), reports.last_mut()) {
                last.issues.push(issue);
            }
        } else if line.starts_with("Notice: ") {
            if let (Some(issue), Some(last)) = (parse_issue(line, true), reports.last_mut()) {
                last.issues.push(issue);
            }
        }
    }
    Ok(reports)
}

// ---------------------------------------------------------------------------
// rendering
// ---------------------------------------------------------------------------

fn boxed(lines: &[String], color: fn(&str) -> ColoredString) {
    println!("{}", color("╭────────────────────────────────────────────────────────────╮").bold());
    for l in lines {
        println!("{} {} {}", color("│").bold(), pad_str(l, 58, Alignment::Center, None), color("│").bold());
    }
    println!("{}", color("╰────────────────────────────────────────────────────────────╯").bold());
}

fn check(paths: Vec<PathBuf>, lang: Lang) -> Result<i32> {
    let t = ui(lang);
    let paths = if paths.is_empty() { vec![PathBuf::from(".")] } else { paths };

    let reports = match run_norminette(&paths) {
        Ok(r) => r,
        Err(_) => {
            println!("\n{} {}", "✗".red().bold(), t.norminette_missing);
            println!("  {}\n", t.install_hint.dimmed());
            return Ok(2);
        }
    };

    println!();
    let title = format!("BetterNorminette v{}", env!("CARGO_PKG_VERSION"));
    boxed(&[title.clone(), t.subtitle.to_string()], |s| s.cyan());
    println!();

    if reports.is_empty() {
        println!("{} {}\n", "⚠".yellow().bold(), t.no_files);
        return Ok(0);
    }

    let mut ok_count = 0usize;
    let mut err_count = 0usize;
    let mut ko_files = 0usize;

    for rep in &reports {
        if rep.ok {
            ok_count += 1;
            continue;
        }
        ko_files += 1;
        println!(" ▶ {}", rep.path.bold());
        for issue in &rep.issues {
            let msg = translate(&issue.code, lang).unwrap_or(&issue.original);
            let badge = if issue.is_notice {
                format!("⚠ {}", t.notice).yellow().bold()
            } else {
                "✗".red().bold()
            };
            err_count += !issue.is_notice as usize;
            let pos = format!("{} {}, {} {}", t.line, issue.line, t.col, issue.col);
            println!("   {} {}  {}  {}",
                badge,
                pad_str(&pos, 18, Alignment::Left, None).dimmed(),
                pad_str(&issue.code, 22, Alignment::Left, None).cyan(),
                msg);
        }
        println!();
    }

    if ok_count > 0 {
        let label = if ok_count == 1 { t.file_ok } else { t.files_ok };
        println!(" {} {} {}\n", "✓".green().bold(), ok_count.to_string().green().bold(), label);
    }

    if ko_files == 0 {
        boxed(&[t.summary_clean.to_string()], |s| s.green());
        println!();
        Ok(0)
    } else {
        let s = format!("✗ {} {} {} {}", err_count, t.summary_errors, ko_files,
            if ko_files == 1 { "file" } else { "files" });
        // keep the count sentence simple across languages
        let s = match lang {
            Lang::En => s,
            Lang::Fr => format!("✗ {} {} {} fichier(s)", err_count, t.summary_errors, ko_files),
            Lang::Es => format!("✗ {} {} {} archivo(s)", err_count, t.summary_errors, ko_files),
        };
        boxed(&[s], |s| s.red());
        println!();
        Ok(1)
    }
}

// ---------------------------------------------------------------------------
// help / update / uninstall
// ---------------------------------------------------------------------------

fn print_help() {
    let v = env!("CARGO_PKG_VERSION");
    println!();
    boxed(&[format!("BetterNorminette v{}", v),
            "norminette multilingue (en · fr · es)".to_string()], |s| s.cyan());
    println!();
    let entry = |cmd: &str, w: usize, desc: &str| {
        println!("  {}  {}", pad_str(cmd, w, Alignment::Left, None).bold(), desc.dimmed());
    };
    println!("{}", "USAGE".bold().yellow());
    entry("better-norminette [fichiers|dossiers]", 40, "vérifie la norme (défaut : dossier courant)");
    entry("better-norminette -l fr src/", 40, "langue ponctuelle pour ce run");
    println!();
    println!("{}", "COMMANDES".bold().yellow());
    entry("lang <en|fr|es>", 18, "définit la langue par défaut");
    entry("update", 18, "met à jour vers la dernière version");
    entry("uninstall", 18, "désinstalle better-norminette");
    entry("version", 18, "affiche la version");
    entry("help", 18, "affiche cette aide");
    println!();
    println!("{}", "ENV".bold().yellow());
    entry("BETTERNORMINETTE_LANG=es", 26, "force la langue");
    entry("BETTERNORMINETTE_NO_UPDATE=1", 26, "désactive la vérification de mise à jour");
    println!();
}

fn run_update(verbose: bool) -> Result<()> {
    let cmd = format!(
        "curl -fsSL --retry 3 https://raw.githubusercontent.com/{}/main/install.sh | sh", REPO);
    let status = if verbose {
        Command::new("sh").arg("-c").arg(&cmd).status()?
    } else {
        Command::new("sh").arg("-c").arg(&cmd)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()?
    };
    if !status.success() {
        anyhow::bail!("update failed");
    }
    Ok(())
}

fn install_root() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    let root = PathBuf::from(home).join(".betternorminette");
    let exe = std::env::current_exe().ok()?.canonicalize().ok()?;
    if exe.starts_with(root.canonicalize().ok()?) { Some(root) } else { None }
}

fn latest_version() -> Option<String> {
    let out = Command::new("curl")
        .args(["-fsSLI", "--retry", "2", "--retry-all-errors",
               "-o", "/dev/null", "-w", "%{url_effective}", "--max-time", "8",
               &format!("https://github.com/{}/releases/latest", REPO)])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let url = String::from_utf8_lossy(&out.stdout);
    let tag = url.trim().rsplit('/').next()?.trim_start_matches('v').to_string();
    if tag.is_empty() || tag == "latest" { None } else { Some(tag) }
}

fn parse_semver(v: &str) -> (u64, u64, u64) {
    let mut it = v.split('.').map(|p| p.trim().parse::<u64>().unwrap_or(0));
    (it.next().unwrap_or(0), it.next().unwrap_or(0), it.next().unwrap_or(0))
}

fn maybe_auto_update() {
    if std::env::var("BETTERNORMINETTE_NO_UPDATE").is_ok() {
        return;
    }
    let Some(root) = install_root() else { return };
    let stamp = root.join(".last_update_check");
    if let Ok(meta) = fs::metadata(&stamp) {
        if let Ok(modified) = meta.modified() {
            if let Ok(age) = modified.elapsed() {
                if age.as_secs() < 24 * 3600 {
                    return;
                }
            }
        }
    }
    let _ = fs::write(&stamp, "");

    let Some(latest) = latest_version() else { return };
    let current = env!("CARGO_PKG_VERSION");
    if parse_semver(&latest) <= parse_semver(current) {
        return;
    }

    use std::io::{IsTerminal, Write};
    if !std::io::stdin().is_terminal() {
        println!("{} v{} available (current v{}) — run {} to upgrade\n",
            "⟳".cyan().bold(), latest, current, "better-norminette update".bold());
        return;
    }
    print!("{} v{} available (current v{}). Update now? [Y/n] ",
        "⟳".cyan().bold(), latest, current);
    let _ = std::io::stdout().flush();
    let mut answer = String::new();
    let _ = std::io::stdin().read_line(&mut answer);
    let answer = answer.trim().to_lowercase();
    if !(answer.is_empty() || answer == "y" || answer == "yes" || answer == "o" || answer == "oui" || answer == "s" || answer == "si" || answer == "sí") {
        println!("{} skipped — run {} whenever you want\n",
            "▸".dimmed(), "better-norminette update".bold());
        return;
    }
    println!("{} updating to v{}...", "⟳".cyan().bold(), latest);
    if run_update(false).is_err() {
        println!("{} update failed, run {} manually\n",
            "⚠".yellow(), "better-norminette update".bold());
        return;
    }
    println!("{} updated to v{}\n", "✓".green().bold(), latest);

    let exe = root.join("better-norminette");
    use std::os::unix::process::CommandExt;
    let args: Vec<String> = std::env::args().skip(1).collect();
    let _ = Command::new(exe)
        .args(args)
        .env("BETTERNORMINETTE_NO_UPDATE", "1")
        .exec();
}

fn main() -> Result<()> {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    if raw.iter().any(|a| a == "-h" || a == "--help") {
        print_help();
        return Ok(());
    }
    if raw.iter().any(|a| a == "-V" || a == "--version") {
        println!("better-norminette {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Lang { lang }) => {
            match Lang::parse(&lang) {
                Some(l) => {
                    save_lang(l)?;
                    println!("{} lang = {}", "✓".green().bold(), l.code().bold());
                }
                None => {
                    println!("{} unknown language: {} (en, fr, es)", "✗".red().bold(), lang);
                    std::process::exit(2);
                }
            }
        }
        Some(Commands::Update) => run_update(true)?,
        Some(Commands::Uninstall) => {
            let cmd = format!(
                "curl -fsSL --retry 3 https://raw.githubusercontent.com/{}/main/uninstall.sh | sh",
                REPO);
            let status = Command::new("sh").arg("-c").arg(&cmd).status()?;
            if !status.success() {
                anyhow::bail!("uninstall failed");
            }
        }
        Some(Commands::Version) => println!("better-norminette {}", env!("CARGO_PKG_VERSION")),
        Some(Commands::Help) => print_help(),
        None => {
            maybe_auto_update();
            let lang = resolve_lang(&cli.lang);
            let code = check(cli.paths, lang)?;
            std::process::exit(code);
        }
    }
    Ok(())
}
