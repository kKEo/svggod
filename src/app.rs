//! The application: arguments in, one command run, its output printed, an exit code out.

use std::io::{IsTerminal, Write};

use clap::Parser;
use clap::error::{ContextKind, ContextValue, ErrorKind};

use crate::cli::{self, Cli, Command, Format};
use crate::envelope::{AppError, EXIT_INTERNAL, EXIT_OK, Output};
use crate::{commands, config, mcp, util};

fn default_format() -> Format {
    if std::io::stdout().is_terminal() { Format::Text } else { Format::Json }
}

/// Best guess at --format when the arguments themselves did not parse.
fn format_from_args(args: &[String]) -> Format {
    for (i, a) in args.iter().enumerate() {
        let v = match a.strip_prefix("--format=") {
            Some(v) => Some(v),
            None if a == "--format" => args.get(i + 1).map(|s| s.as_str()),
            None => None,
        };
        match v {
            Some("json") => return Format::Json,
            Some("text") => return Format::Text,
            _ => {}
        }
    }
    default_format()
}

fn emit(format: Format, out: &Output) {
    let mut stdout = std::io::stdout().lock();
    let _ = match (&out.raw, format) {
        (Some(raw), _) => stdout.write_all(raw),
        (None, Format::Json) => writeln!(stdout, "{}", out.json),
        (None, Format::Text) => write!(stdout, "{}", out.text),
    };
    let _ = stdout.flush();
}

fn fail(format: Format, command: &str, err: &AppError) -> i32 {
    match format {
        Format::Json => {
            let mut stdout = std::io::stdout().lock();
            let _ = writeln!(stdout, "{}", err.to_json(command));
        }
        Format::Text => eprintln!("{}", err.to_text()),
    }
    err.exit
}

pub(crate) fn usage_error(e: clap::Error, args: &[String]) -> AppError {
    let rendered = e.render().to_string();
    let first = rendered.lines().next().unwrap_or("invalid arguments").trim_start_matches("error: ").trim().to_string();
    let message = if first.ends_with('.') { first } else { format!("{first}.") };
    let ctx_str = |kind: ContextKind| match e.get(kind) {
        Some(ContextValue::String(s)) => Some(s.clone()),
        Some(ContextValue::Strings(v)) => v.first().cloned(),
        _ => None,
    };
    let mut suggestion = ctx_str(ContextKind::SuggestedArg)
        .or_else(|| ctx_str(ContextKind::SuggestedSubcommand))
        .or_else(|| ctx_str(ContextKind::SuggestedValue))
        .map(|s| format!("Did you mean '{s}'?"));
    let code = match e.kind() {
        ErrorKind::UnknownArgument => "unknown-flag",
        ErrorKind::InvalidSubcommand => "unknown-command",
        ErrorKind::MissingRequiredArgument | ErrorKind::MissingSubcommand | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => "missing-argument",
        ErrorKind::ArgumentConflict => "argument-conflict",
        _ => "invalid-argument",
    };
    let mut message = message;
    if e.kind() == ErrorKind::InvalidSubcommand
        && let Some(cmd) = args.iter().find(|a| cli::PLANNED.contains(&a.as_str()))
    {
        message = format!("The '{cmd}' command is specified but not built yet; this build has {}.", cli::COMMANDS.join(", "));
        suggestion = None;
    }
    if e.kind() == ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand {
        message = format!("No command given; use one of {}.", cli::COMMANDS.join(", "));
    }
    if suggestion.is_none() && code != "unknown-command" {
        suggestion = Some("Run with --help to see the accepted arguments.".into());
    }
    AppError::usage(code, message).suggest(suggestion)
}

/// What differs between a command typed in a shell and one arriving as a tool call.
pub(crate) struct Env {
    pub color: bool,
    /// Config discovery stops at this directory.
    pub ceiling: Option<std::path::PathBuf>,
    /// Edit operations supplied as data in place of an --ops file.
    pub inline_ops: Option<serde_json::Value>,
    /// A diagram spec supplied as text in place of a spec file.
    pub inline_spec: Option<String>,
}

/// Run one parsed command and hand back its output; nothing is printed here.
pub(crate) fn execute(cli: &Cli, env: &Env) -> Result<Output, AppError> {
    let profile = match &cli.global.profile {
        Some(p) => Some(config::Profile::parse(p).ok_or_else(|| AppError::usage("unknown-profile", format!("Unknown profile '{p}'.")).suggest(util::did_you_mean(p, config::PROFILES.iter().copied())))?),
        None => None,
    };
    let paint = commands::validate::Paint { color: env.color };
    let mut loader = match &cli.command {
        Command::Validate { .. } | Command::Edit { .. } | Command::Fix { .. } | Command::Render { .. } | Command::Preview { .. } | Command::Docs { .. } | Command::Theme { .. } => Some(config::Loader::new(cli.global.config.as_deref(), cli.global.no_config, env.ceiling.clone())?),
        _ => None,
    };
    let out = match &cli.command {
        Command::Validate { files, rules, skip, fail_on, max_findings } => commands::validate::run(files, rules, skip, *fail_on, *max_findings, loader.as_mut().unwrap(), profile, &paint),
        Command::Edit { file, set, ops, dry_run, output, in_place } => commands::edit::run(file, set, ops.as_deref(), env.inline_ops.as_ref(), *dry_run, output.as_deref(), *in_place, loader.as_mut().unwrap(), profile, &paint),
        Command::Fix { file, rules, allow_unsafe, dry_run, output, in_place } => commands::fix::run(file, rules, *allow_unsafe, *dry_run, output.as_deref(), *in_place, loader.as_mut().unwrap(), profile, &paint),
        Command::Render { spec, output, theme, effort, interactive, frames, embed_spec, check, previous, no_validate, force, text_as_paths } => commands::render::run(spec, env.inline_spec.as_deref(), output, theme.as_deref(), effort.as_deref(), *interactive, *frames, *embed_spec, *check, previous.as_deref(), *no_validate, *force, *text_as_paths, loader.as_mut().unwrap(), profile, &paint),
        Command::Preview { file, output, width, background, annotate, crop, step, force } => commands::preview::run(file, output, *width, *background, *annotate, crop.as_deref(), *step, *force, loader.as_mut().unwrap(), profile, &paint),
        Command::Inspect { file, select, depth, fields } => commands::inspect::run(file, select.as_deref(), *depth, fields),
        Command::Schema { name } => commands::schema::run(*name),
        Command::Rules { explain } => commands::rules::run(explain.as_deref()),
        Command::Examples { name } => commands::examples::run(name.as_deref()),
        Command::Describe { file, short } => commands::describe::run(file, *short),
        Command::Docs { paths, check, assets, theme, force, interactive } => commands::docs::run(paths, *check, assets, theme.as_deref(), *force, *interactive, loader.as_mut().unwrap(), profile, &paint),
        Command::Theme { action } => match action {
            cli::ThemeAction::List => commands::theme::list(),
            cli::ThemeAction::Init { dir, from, force } => commands::theme::init(dir, from, *force),
            cli::ThemeAction::Check { dir } => commands::theme::check(dir, loader.as_mut().unwrap(), profile, &paint),
        },
        Command::Mcp { .. } => Err(AppError::usage("invalid-argument", "The mcp server cannot be started from inside a tool call.")),
    }?;
    if let Some(loader) = &loader {
        LOADED.with(|l| *l.borrow_mut() = loader.loaded.clone());
    }
    Ok(out)
}

thread_local! {
    /// Config files the last command read, for the stderr log.
    static LOADED: std::cell::RefCell<Vec<std::path::PathBuf>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn dispatch(args: &[String]) -> i32 {
    let cli = match Cli::try_parse_from(args) {
        Ok(c) => c,
        Err(e) => {
            if matches!(e.kind(), ErrorKind::DisplayHelp | ErrorKind::DisplayVersion) {
                print!("{}", e.render());
                return EXIT_OK;
            }
            let command = args.iter().skip(1).find(|a| cli::COMMANDS.contains(&a.as_str())).map(|s| s.as_str()).unwrap_or("svggod");
            return fail(format_from_args(args), command, &usage_error(e, args));
        }
    };
    let format = cli.global.format.unwrap_or_else(default_format);
    let name = cli.command.name();
    if let Command::Mcp { root_path } = &cli.command {
        // The server owns stdout from here on; a start-up failure is reported as JSON.
        return match mcp::serve(root_path) {
            Ok(()) => EXIT_OK,
            Err(e) => fail(Format::Json, name, &e),
        };
    }
    let color = format == Format::Text && std::io::stdout().is_terminal() && !cli.global.no_color && std::env::var_os("NO_COLOR").is_none_or(|v| v.is_empty());
    let result = execute(&cli, &Env { color, ceiling: None, inline_ops: None, inline_spec: None });
    if !cli.global.quiet {
        LOADED.with(|l| {
            for p in l.borrow().iter() {
                eprintln!("svggod: config {}", p.display());
            }
        });
    }
    match result {
        Ok(out) => {
            emit(format, &out);
            out.exit
        }
        Err(e) => fail(format, name, &e),
    }
}

/// Run the command line in `args` (the program name first) and return the exit code.
///
/// Output goes to stdout and stderr as the command line contract says. A panic is
/// a bug: it is reported in the contract's shape instead of a backtrace.
pub fn run(args: &[String]) -> i32 {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let outcome = std::panic::catch_unwind(|| dispatch(args));
    std::panic::set_hook(hook);
    match outcome {
        Ok(code) => code,
        Err(payload) => {
            let detail = payload.downcast_ref::<&str>().map(|s| s.to_string()).or_else(|| payload.downcast_ref::<String>().cloned()).unwrap_or_else(|| "unknown panic".into());
            let err = AppError::new(EXIT_INTERNAL, "internal-error", format!("svggod hit a bug ({detail}). Please report it with the command and the input file."));
            let command = args.iter().skip(1).find(|a| cli::COMMANDS.contains(&a.as_str())).map(|s| s.as_str()).unwrap_or("svggod");
            fail(format_from_args(args), command, &err)
        }
    }
}

/// The `svggod` program: run the process's own arguments and exit with the code.
pub fn main() -> ! {
    let args: Vec<String> = std::env::args_os().map(|a| a.to_string_lossy().into_owned()).collect();
    std::process::exit(run(&args))
}
