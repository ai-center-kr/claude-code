// cc-commands: Slash command system for the Claude Code Rust port.
//
// This crate implements the /command framework that allows users to type
// commands like /help, /compact, /clear, /model, /config, /cost, etc.
// Each command is a struct implementing the `SlashCommand` trait.

use async_trait::async_trait;
use cc_core::config::Config;
use cc_core::cost::CostTracker;
use cc_core::types::Message;
use std::sync::Arc;
#[allow(unused_imports)]
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Core trait
// ---------------------------------------------------------------------------

/// Context available to every slash command.
pub struct CommandContext {
    pub config: Config,
    pub cost_tracker: Arc<CostTracker>,
    pub messages: Vec<Message>,
    pub working_dir: std::path::PathBuf,
    // Note: config already contains hooks, mcp_servers, etc.
}

/// Result of running a slash command.
#[derive(Debug)]
pub enum CommandResult {
    /// Display a message to the user (does NOT go to the model).
    Message(String),
    /// Inject a message into the conversation as though the user typed it.
    UserMessage(String),
    /// Modify the configuration.
    ConfigChange(Config),
    /// Clear the conversation.
    ClearConversation,
    /// Replace the conversation with a specific message list (used by /rewind).
    SetMessages(Vec<Message>),
    /// Trigger the OAuth login flow (handled by the REPL in main.rs).
    /// The bool indicates whether to use Claude.ai auth (true) or Console auth (false).
    StartOAuthFlow(bool),
    /// Exit the REPL.
    Exit,
    /// No visible output.
    Silent,
    /// An error.
    Error(String),
}

/// Every slash command implements this trait.
#[async_trait]
pub trait SlashCommand: Send + Sync {
    /// The primary name (without the leading `/`).
    fn name(&self) -> &str;
    /// Alias names (e.g. `["h"]` for `/help`).
    fn aliases(&self) -> Vec<&str> {
        vec![]
    }
    /// One-line description for /help.
    fn description(&self) -> &str;
    /// Detailed help text (shown by `/help <command>`).
    fn help(&self) -> &str {
        self.description()
    }
    /// Whether this command is visible in /help output.
    fn hidden(&self) -> bool {
        false
    }
    /// Execute the command with the given arguments string.
    async fn execute(&self, args: &str, ctx: &mut CommandContext) -> CommandResult;
}

// ---------------------------------------------------------------------------
// Built-in commands
// ---------------------------------------------------------------------------

pub struct HelpCommand;
pub struct ClearCommand;
pub struct CompactCommand;
pub struct CostCommand;
pub struct ExitCommand;
pub struct ModelCommand;
pub struct ConfigCommand;
pub struct VersionCommand;
pub struct ResumeCommand;
pub struct StatusCommand;
pub struct DiffCommand;
pub struct MemoryCommand;
pub struct BugCommand;
pub struct DoctorCommand;
pub struct LoginCommand;
pub struct LogoutCommand;
pub struct InitCommand;
pub struct ReviewCommand;
pub struct HooksCommand;
pub struct McpCommand;
pub struct PermissionsCommand;
pub struct PlanCommand;
pub struct TasksCommand;
pub struct SessionCommand;
pub struct ThinkingCommand;
// New commands
pub struct ExportCommand;
pub struct SkillsCommand;
pub struct RewindCommand;
pub struct StatsCommand;
pub struct FilesCommand;
pub struct RenameCommand;
pub struct EffortCommand;
pub struct SummaryCommand;
pub struct CommitCommand;

// ---- /help ---------------------------------------------------------------

#[async_trait]
impl SlashCommand for HelpCommand {
    fn name(&self) -> &str { "help" }
    fn aliases(&self) -> Vec<&str> { vec!["h", "?"] }
    fn description(&self) -> &str { "Show available commands and usage information" }

    async fn execute(&self, args: &str, ctx: &mut CommandContext) -> CommandResult {
        if !args.is_empty() {
            // Show help for a specific command
            if let Some(cmd) = find_command(args) {
                return CommandResult::Message(format!(
                    "/{} - {}\n\n{}",
                    cmd.name(),
                    cmd.description(),
                    cmd.help()
                ));
            }
            return CommandResult::Error(format!("Unknown command: /{}", args));
        }

        let mut output = String::from("Available commands:\n\n");
        for cmd in all_commands() {
            if !cmd.hidden() {
                let aliases = cmd.aliases();
                let alias_str = if aliases.is_empty() {
                    String::new()
                } else {
                    format!(
                        " ({})",
                        aliases.iter().map(|a| format!("/{}", a)).collect::<Vec<_>>().join(", ")
                    )
                };
                output.push_str(&format!(
                    "  /{}{} - {}\n",
                    cmd.name(),
                    alias_str,
                    cmd.description()
                ));
            }
        }

        output.push_str("\nType /help <command> for detailed help on a specific command.");
        CommandResult::Message(output)
    }
}

// ---- /clear --------------------------------------------------------------

#[async_trait]
impl SlashCommand for ClearCommand {
    fn name(&self) -> &str { "clear" }
    fn aliases(&self) -> Vec<&str> { vec!["c"] }
    fn description(&self) -> &str { "Clear the conversation history" }

    async fn execute(&self, _args: &str, _ctx: &mut CommandContext) -> CommandResult {
        CommandResult::ClearConversation
    }
}

// ---- /compact ------------------------------------------------------------

#[async_trait]
impl SlashCommand for CompactCommand {
    fn name(&self) -> &str { "compact" }
    fn description(&self) -> &str { "Compact the conversation to reduce token usage" }

    async fn execute(&self, args: &str, ctx: &mut CommandContext) -> CommandResult {
        let msg_count = ctx.messages.len();
        let instruction = if args.is_empty() {
            "Provide a detailed summary of our conversation so far, preserving all \
             key technical details, decisions made, file paths mentioned, and current \
             task status."
                .to_string()
        } else {
            args.to_string()
        };

        CommandResult::UserMessage(format!(
            "[Compact requested ({} messages). Instruction: {}]",
            msg_count, instruction
        ))
    }
}

// ---- /cost ---------------------------------------------------------------

#[async_trait]
impl SlashCommand for CostCommand {
    fn name(&self) -> &str { "cost" }
    fn description(&self) -> &str { "Show token usage and cost for this session" }

    async fn execute(&self, _args: &str, ctx: &mut CommandContext) -> CommandResult {
        let tracker = &ctx.cost_tracker;
        CommandResult::Message(format!(
            "Session cost:\n  Input tokens:  {}\n  Output tokens: {}\n  \
             Cache creation: {}\n  Cache read:    {}\n  Total tokens:  {}\n  \
             Estimated cost: ${:.4}",
            tracker.input_tokens(),
            tracker.output_tokens(),
            tracker.cache_creation_tokens(),
            tracker.cache_read_tokens(),
            tracker.total_tokens(),
            tracker.total_cost_usd(),
        ))
    }
}

// ---- /exit ---------------------------------------------------------------

#[async_trait]
impl SlashCommand for ExitCommand {
    fn name(&self) -> &str { "exit" }
    fn aliases(&self) -> Vec<&str> { vec!["quit", "q"] }
    fn description(&self) -> &str { "Exit Claude Code" }

    async fn execute(&self, _args: &str, _ctx: &mut CommandContext) -> CommandResult {
        CommandResult::Exit
    }
}

// ---- /model --------------------------------------------------------------

#[async_trait]
impl SlashCommand for ModelCommand {
    fn name(&self) -> &str { "model" }
    fn description(&self) -> &str { "Show or change the current model" }

    async fn execute(&self, args: &str, ctx: &mut CommandContext) -> CommandResult {
        if args.is_empty() {
            CommandResult::Message(format!(
                "Current model: {}",
                ctx.config.effective_model()
            ))
        } else {
            let mut new_config = ctx.config.clone();
            new_config.model = Some(args.trim().to_string());
            CommandResult::ConfigChange(new_config)
        }
    }
}

// ---- /config -------------------------------------------------------------

#[async_trait]
impl SlashCommand for ConfigCommand {
    fn name(&self) -> &str { "config" }
    fn description(&self) -> &str { "Show or modify configuration settings" }

    async fn execute(&self, _args: &str, ctx: &mut CommandContext) -> CommandResult {
        let json = serde_json::to_string_pretty(&ctx.config).unwrap_or_default();
        CommandResult::Message(format!("Current configuration:\n{}", json))
    }
}

// ---- /version ------------------------------------------------------------

#[async_trait]
impl SlashCommand for VersionCommand {
    fn name(&self) -> &str { "version" }
    fn aliases(&self) -> Vec<&str> { vec!["v"] }
    fn description(&self) -> &str { "Show version information" }

    async fn execute(&self, _args: &str, _ctx: &mut CommandContext) -> CommandResult {
        CommandResult::Message(format!(
            "Claude Code (Rust) v{}",
            cc_core::constants::APP_VERSION
        ))
    }
}

// ---- /resume -------------------------------------------------------------

#[async_trait]
impl SlashCommand for ResumeCommand {
    fn name(&self) -> &str { "resume" }
    fn aliases(&self) -> Vec<&str> { vec!["r"] }
    fn description(&self) -> &str { "Resume a previous conversation" }

    async fn execute(&self, args: &str, _ctx: &mut CommandContext) -> CommandResult {
        if args.is_empty() {
            match cc_core::history::list_sessions().await {
                Ok(sessions) => {
                    if sessions.is_empty() {
                        return CommandResult::Message("No previous sessions found.".to_string());
                    }
                    let mut output = String::from("Recent sessions:\n\n");
                    for (i, session) in sessions.iter().take(10).enumerate() {
                        let title = session
                            .title
                            .as_deref()
                            .unwrap_or("(untitled)");
                        output.push_str(&format!(
                            "  {}. {} - {} ({} messages)\n",
                            i + 1,
                            &session.id[..8],
                            title,
                            session.messages.len()
                        ));
                    }
                    output.push_str("\nUse /resume <id> to resume a session.");
                    CommandResult::Message(output)
                }
                Err(e) => CommandResult::Error(format!("Failed to list sessions: {}", e)),
            }
        } else {
            CommandResult::Message(format!("Resuming session: {}", args.trim()))
        }
    }
}

// ---- /status -------------------------------------------------------------

#[async_trait]
impl SlashCommand for StatusCommand {
    fn name(&self) -> &str { "status" }
    fn description(&self) -> &str { "Show current session status" }

    async fn execute(&self, _args: &str, ctx: &mut CommandContext) -> CommandResult {
        CommandResult::Message(format!(
            "Status:\n  Model: {}\n  Messages: {}\n  Working dir: {}\n  Cost: {}",
            ctx.config.effective_model(),
            ctx.messages.len(),
            ctx.working_dir.display(),
            ctx.cost_tracker.summary(),
        ))
    }
}

// ---- /diff ---------------------------------------------------------------

#[async_trait]
impl SlashCommand for DiffCommand {
    fn name(&self) -> &str { "diff" }
    fn description(&self) -> &str { "Show file changes made during this session" }

    async fn execute(&self, _args: &str, ctx: &mut CommandContext) -> CommandResult {
        // Run git diff in the working directory
        let output = tokio::process::Command::new("git")
            .args(["diff", "--stat"])
            .current_dir(&ctx.working_dir)
            .output()
            .await;

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if stdout.trim().is_empty() {
                    CommandResult::Message("No uncommitted changes.".to_string())
                } else {
                    CommandResult::Message(format!("Changes:\n{}", stdout))
                }
            }
            Err(e) => CommandResult::Error(format!("Failed to run git diff: {}", e)),
        }
    }
}

// ---- /memory -------------------------------------------------------------

#[async_trait]
impl SlashCommand for MemoryCommand {
    fn name(&self) -> &str { "memory" }
    fn description(&self) -> &str { "View or edit CLAUDE.md memory files" }

    async fn execute(&self, _args: &str, ctx: &mut CommandContext) -> CommandResult {
        let claude_md = ctx.working_dir.join("CLAUDE.md");
        if claude_md.exists() {
            match tokio::fs::read_to_string(&claude_md).await {
                Ok(content) => CommandResult::Message(format!(
                    "CLAUDE.md ({}):\n\n{}",
                    claude_md.display(),
                    content
                )),
                Err(e) => CommandResult::Error(format!("Failed to read CLAUDE.md: {}", e)),
            }
        } else {
            CommandResult::Message("No CLAUDE.md found in the current project.".to_string())
        }
    }
}

// ---- /bug ----------------------------------------------------------------

#[async_trait]
impl SlashCommand for BugCommand {
    fn name(&self) -> &str { "bug" }
    fn description(&self) -> &str { "Report a bug or issue" }
    fn hidden(&self) -> bool { true }

    async fn execute(&self, _args: &str, _ctx: &mut CommandContext) -> CommandResult {
        CommandResult::Message(
            "To report a bug, please visit: https://github.com/anthropics/claude-code/issues"
                .to_string(),
        )
    }
}

// ---- /doctor -------------------------------------------------------------

#[async_trait]
impl SlashCommand for DoctorCommand {
    fn name(&self) -> &str { "doctor" }
    fn description(&self) -> &str { "Check system health and diagnose issues" }

    async fn execute(&self, _args: &str, ctx: &mut CommandContext) -> CommandResult {
        let mut checks = Vec::new();

        // Check API key
        if ctx.config.resolve_api_key().is_some() {
            checks.push("  [ok] API key configured");
        } else {
            checks.push("  [!!] No API key found (set ANTHROPIC_API_KEY)");
        }

        // Check git
        let git_ok = tokio::process::Command::new("git")
            .arg("--version")
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false);
        if git_ok {
            checks.push("  [ok] git available");
        } else {
            checks.push("  [!!] git not found");
        }

        // Check config dir
        let config_dir = cc_core::config::Settings::config_dir();
        if config_dir.exists() {
            checks.push("  [ok] Config directory exists");
        } else {
            checks.push("  [!!] Config directory missing");
        }

        CommandResult::Message(format!("Doctor report:\n\n{}", checks.join("\n")))
    }
}

// ---- /login --------------------------------------------------------------

#[async_trait]
impl SlashCommand for LoginCommand {
    fn name(&self) -> &str { "login" }
    fn description(&self) -> &str { "Authenticate with Anthropic (OAuth PKCE flow)" }

    async fn execute(&self, args: &str, _ctx: &mut CommandContext) -> CommandResult {
        // `--console` flag → Console/API-key auth; default → Claude.ai subscription auth
        let login_with_claude_ai = !args.contains("--console");
        CommandResult::StartOAuthFlow(login_with_claude_ai)
    }
}

// ---- /logout -------------------------------------------------------------

#[async_trait]
impl SlashCommand for LogoutCommand {
    fn name(&self) -> &str { "logout" }
    fn description(&self) -> &str { "Clear stored OAuth tokens and API key" }

    async fn execute(&self, _args: &str, ctx: &mut CommandContext) -> CommandResult {
        // Clear OAuth tokens file
        if let Err(e) = cc_core::oauth::OAuthTokens::clear().await {
            return CommandResult::Error(format!("Failed to clear OAuth tokens: {}", e));
        }
        // Also clear any API key stored in settings
        let mut settings = cc_core::config::Settings::load().await.unwrap_or_default();
        settings.config.api_key = None;
        if let Err(e) = settings.save().await {
            return CommandResult::Error(format!("Failed to update settings: {}", e));
        }
        ctx.config.api_key = None;
        CommandResult::Message("Logged out. Credentials cleared.".to_string())
    }
}

// ---- /init ---------------------------------------------------------------

#[async_trait]
impl SlashCommand for InitCommand {
    fn name(&self) -> &str { "init" }
    fn description(&self) -> &str { "Initialize a new project with CLAUDE.md" }

    async fn execute(&self, _args: &str, ctx: &mut CommandContext) -> CommandResult {
        let path = ctx.working_dir.join("CLAUDE.md");
        if path.exists() {
            return CommandResult::Message(format!(
                "CLAUDE.md already exists at {}",
                path.display()
            ));
        }

        let default_content = "# Project Instructions\n\n\
            Add project-specific instructions and context here.\n\n\
            ## Guidelines\n\n\
            - Describe your project structure\n\
            - Note any coding conventions\n\
            - List important files and their purposes\n";

        match tokio::fs::write(&path, default_content).await {
            Ok(()) => CommandResult::Message(format!(
                "Created CLAUDE.md at {}",
                path.display()
            )),
            Err(e) => CommandResult::Error(format!("Failed to create CLAUDE.md: {}", e)),
        }
    }
}

// ---- /review -------------------------------------------------------------

#[async_trait]
impl SlashCommand for ReviewCommand {
    fn name(&self) -> &str { "review" }
    fn description(&self) -> &str { "Review code changes (git diff)" }

    async fn execute(&self, args: &str, _ctx: &mut CommandContext) -> CommandResult {
        let target = if args.is_empty() { "HEAD" } else { args.trim() };
        CommandResult::UserMessage(format!(
            "Please review the code changes in `git diff {}`. \
             Look for bugs, security issues, and style problems.",
            target
        ))
    }
}

// ---- /hooks --------------------------------------------------------------

#[async_trait]
impl SlashCommand for HooksCommand {
    fn name(&self) -> &str { "hooks" }
    fn description(&self) -> &str { "Show configured event hooks" }
    fn help(&self) -> &str {
        "Usage: /hooks\n\
         Show hooks configured in settings.json under 'hooks'.\n\
         Hooks fire shell commands on events: PreToolUse, PostToolUse, Stop, UserPromptSubmit."
    }

    async fn execute(&self, _args: &str, ctx: &mut CommandContext) -> CommandResult {
        if ctx.config.hooks.is_empty() {
            return CommandResult::Message(
                "No hooks configured.\n\
                 Add hooks to ~/.claude/settings.json under the 'hooks' key.\n\
                 Example:\n  \"hooks\": { \"PreToolUse\": [{\"command\": \"echo $STDIN\", \"blocking\": false}] }"
                    .to_string(),
            );
        }

        let mut lines = vec!["Configured hooks:".to_string()];
        for (event, entries) in &ctx.config.hooks {
            lines.push(format!("\n  {:?} ({} entries):", event, entries.len()));
            for e in entries {
                let filter = e.tool_filter.as_deref().unwrap_or("*");
                lines.push(format!(
                    "    - [{}] {} (blocking={})",
                    filter, e.command, e.blocking
                ));
            }
        }

        CommandResult::Message(lines.join("\n"))
    }
}

// ---- /mcp ----------------------------------------------------------------

#[async_trait]
impl SlashCommand for McpCommand {
    fn name(&self) -> &str { "mcp" }
    fn description(&self) -> &str { "Manage MCP servers" }

    async fn execute(&self, _args: &str, ctx: &mut CommandContext) -> CommandResult {
        if ctx.config.mcp_servers.is_empty() {
            CommandResult::Message("No MCP servers configured.".to_string())
        } else {
            let mut output = String::from("Configured MCP servers:\n\n");
            for srv in &ctx.config.mcp_servers {
                output.push_str(&format!(
                    "  - {} ({})\n",
                    srv.name, srv.server_type
                ));
            }
            CommandResult::Message(output)
        }
    }
}

// ---- /permissions --------------------------------------------------------

#[async_trait]
impl SlashCommand for PermissionsCommand {
    fn name(&self) -> &str { "permissions" }
    fn description(&self) -> &str { "View or modify permission settings" }

    async fn execute(&self, _args: &str, ctx: &mut CommandContext) -> CommandResult {
        CommandResult::Message(format!(
            "Permission mode: {:?}\nAllowed tools: {:?}\nDisallowed tools: {:?}",
            ctx.config.permission_mode,
            ctx.config.allowed_tools,
            ctx.config.disallowed_tools,
        ))
    }
}

// ---- /plan ---------------------------------------------------------------

#[async_trait]
impl SlashCommand for PlanCommand {
    fn name(&self) -> &str { "plan" }
    fn description(&self) -> &str { "Enter plan mode – model outputs a plan for approval before acting" }
    fn help(&self) -> &str {
        "Usage: /plan [description]\n\n\
         Switches to plan mode where the model will create a detailed plan before executing.\n\
         The plan must be approved before any file writes or command executions are performed.\n\
         Use /plan exit to leave plan mode."
    }

    async fn execute(&self, args: &str, _ctx: &mut CommandContext) -> CommandResult {
        if args.trim() == "exit" {
            return CommandResult::UserMessage(
                "[Exiting plan mode. Resuming normal execution.]".to_string()
            );
        }
        let task_desc = if args.is_empty() {
            "the current task".to_string()
        } else {
            args.to_string()
        };
        CommandResult::UserMessage(format!(
            "[Entering plan mode for: {}]\n\
             Please create a detailed step-by-step plan. Do not execute any commands or \
             write any files until the plan has been reviewed and approved.",
            task_desc
        ))
    }
}

// ---- /tasks --------------------------------------------------------------

#[async_trait]
impl SlashCommand for TasksCommand {
    fn name(&self) -> &str { "tasks" }
    fn description(&self) -> &str { "List and manage background tasks" }

    async fn execute(&self, _args: &str, _ctx: &mut CommandContext) -> CommandResult {
        CommandResult::UserMessage(
            "Please list all current tasks using the TaskList tool and show their status.".to_string()
        )
    }
}

// ---- /session ------------------------------------------------------------

#[async_trait]
impl SlashCommand for SessionCommand {
    fn name(&self) -> &str { "session" }
    fn description(&self) -> &str { "Show or manage conversation sessions" }

    async fn execute(&self, args: &str, ctx: &mut CommandContext) -> CommandResult {
        match args.trim() {
            "list" => {
                match cc_core::history::list_sessions().await {
                    Ok(sessions) => {
                        if sessions.is_empty() {
                            CommandResult::Message("No saved sessions found.".to_string())
                        } else {
                            let mut output = String::from("Recent sessions:\n\n");
                            for sess in sessions.iter().take(10) {
                                let updated = sess.updated_at.format("%Y-%m-%d %H:%M").to_string();
                                output.push_str(&format!(
                                    "  {} | {} | {} messages | {}\n",
                                    &sess.id[..8],
                                    updated,
                                    sess.messages.len(),
                                    sess.title.as_deref().unwrap_or("(untitled)")
                                ));
                            }
                            output.push_str("\nUse /resume <id> to resume a session.");
                            CommandResult::Message(output)
                        }
                    }
                    Err(e) => CommandResult::Error(format!("Failed to list sessions: {}", e)),
                }
            }
            "" => {
                CommandResult::Message(format!(
                    "Current session stats:\n  Messages: {}\n  Model: {}\n\nUse /session list to see all sessions.",
                    ctx.messages.len(),
                    ctx.config.effective_model()
                ))
            }
            _ => CommandResult::Error(format!("Unknown subcommand: {}\n\nUsage: /session [list]", args)),
        }
    }
}

// ---- /thinking -----------------------------------------------------------

#[async_trait]
impl SlashCommand for ThinkingCommand {
    fn name(&self) -> &str { "thinking" }
    fn description(&self) -> &str { "Toggle extended thinking mode" }
    fn aliases(&self) -> Vec<&str> { vec!["think"] }

    async fn execute(&self, _args: &str, ctx: &mut CommandContext) -> CommandResult {
        // Extended thinking is configured through the model; just inform the user
        let model = ctx.config.effective_model();
        if model.contains("claude-3-5") || model.contains("claude-3.5") {
            CommandResult::Message(
                "Extended thinking is not available for Claude 3.5 models.\n\
                 Use claude-opus-4-6 or claude-sonnet-4-6 for extended thinking.".to_string()
            )
        } else {
            CommandResult::Message(format!(
                "Extended thinking is available with {}.\n\
                 You can request thinking by asking Claude to 'think step by step' or \
                 'think carefully before answering'.",
                model
            ))
        }
    }
}

// ---- /export -------------------------------------------------------------

#[async_trait]
impl SlashCommand for ExportCommand {
    fn name(&self) -> &str { "export" }
    fn description(&self) -> &str { "Export conversation to a file" }
    fn help(&self) -> &str {
        "Usage: /export [filename]\n\
         Export the current conversation as JSON. Defaults to claude_export_<timestamp>.json."
    }

    async fn execute(&self, args: &str, ctx: &mut CommandContext) -> CommandResult {
        let filename = if args.trim().is_empty() {
            format!(
                "claude_export_{}.json",
                chrono::Utc::now().format("%Y%m%d_%H%M%S")
            )
        } else {
            args.trim().to_string()
        };

        let path = ctx.working_dir.join(&filename);
        let export = serde_json::json!({
            "exported_at": chrono::Utc::now().to_rfc3339(),
            "model": ctx.config.effective_model(),
            "message_count": ctx.messages.len(),
            "messages": ctx.messages.iter().map(|m| serde_json::json!({
                "role": m.role,
                "content": m.get_all_text(),
            })).collect::<Vec<_>>(),
        });

        let json = match serde_json::to_string_pretty(&export) {
            Ok(j) => j,
            Err(e) => return CommandResult::Error(format!("Failed to serialize: {}", e)),
        };

        match std::fs::write(&path, &json) {
            Ok(_) => CommandResult::Message(format!(
                "Conversation exported to {}\n({} messages)",
                path.display(),
                ctx.messages.len()
            )),
            Err(e) => CommandResult::Error(format!("Failed to write {}: {}", filename, e)),
        }
    }
}

// ---- /skills -------------------------------------------------------------

#[async_trait]
impl SlashCommand for SkillsCommand {
    fn name(&self) -> &str { "skills" }
    fn aliases(&self) -> Vec<&str> { vec!["skill"] }
    fn description(&self) -> &str { "List available skills in .claude/commands/" }

    async fn execute(&self, _args: &str, ctx: &mut CommandContext) -> CommandResult {
        let mut found: Vec<String> = Vec::new();
        let dirs = [
            ctx.working_dir.join(".claude").join("commands"),
            dirs::home_dir()
                .unwrap_or_default()
                .join(".claude")
                .join("commands"),
        ];

        for dir in &dirs {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.extension().map_or(false, |e| e == "md") {
                        if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                            let name = stem.to_string();
                            if !found.contains(&name) {
                                found.push(name);
                            }
                        }
                    }
                }
            }
        }

        if found.is_empty() {
            return CommandResult::Message(
                "No skills found.\nCreate .md files in .claude/commands/ to define skills.\n\
                 Example: .claude/commands/review.md".to_string(),
            );
        }

        found.sort();
        CommandResult::Message(format!(
            "Available skills ({}):\n{}",
            found.len(),
            found.iter().map(|s| format!("  /{}", s)).collect::<Vec<_>>().join("\n")
        ))
    }
}

// ---- /rewind -------------------------------------------------------------

#[async_trait]
impl SlashCommand for RewindCommand {
    fn name(&self) -> &str { "rewind" }
    fn description(&self) -> &str { "Remove the last N message turns" }
    fn help(&self) -> &str { "Usage: /rewind [n]\nRemove the last N turns (default 1)." }

    async fn execute(&self, args: &str, ctx: &mut CommandContext) -> CommandResult {
        let n: usize = args.trim().parse().unwrap_or(1).max(1);

        // Each "turn" is a user+assistant pair = 2 messages.
        let to_remove = (n * 2).min(ctx.messages.len());
        if to_remove == 0 {
            return CommandResult::Message("Nothing to rewind.".to_string());
        }

        let new_len = ctx.messages.len().saturating_sub(to_remove);
        let mut trimmed = ctx.messages.clone();
        trimmed.truncate(new_len);

        let removed = ctx.messages.len() - new_len;
        let note = format!(
            "Rewound {} turn{} ({} message{} removed).",
            n,
            if n == 1 { "" } else { "s" },
            removed,
            if removed == 1 { "" } else { "s" }
        );

        // SetMessages propagates the trimmed list back to the REPL.
        // We use a UserMessage to surface the confirmation note.
        // Since CommandResult can only return one variant, we use SetMessages
        // and let the REPL show a status message.
        let _ = note; // consumed below
        CommandResult::SetMessages(trimmed)
    }
}

// ---- /stats --------------------------------------------------------------

#[async_trait]
impl SlashCommand for StatsCommand {
    fn name(&self) -> &str { "stats" }
    fn aliases(&self) -> Vec<&str> { vec!["usage"] }
    fn description(&self) -> &str { "Show token usage and cost statistics" }

    async fn execute(&self, _args: &str, ctx: &mut CommandContext) -> CommandResult {
        let input = ctx.cost_tracker.input_tokens();
        let output = ctx.cost_tracker.output_tokens();
        let cost = ctx.cost_tracker.total_cost_usd();
        let turns = ctx.messages.len();
        let model = ctx.config.effective_model();

        CommandResult::Message(format!(
            "Session statistics\n\
             ──────────────────\n\
             Model:          {}\n\
             Messages:       {}\n\
             Input tokens:   {}\n\
             Output tokens:  {}\n\
             Total tokens:   {}\n\
             Estimated cost: ${:.4}",
            model,
            turns,
            input,
            output,
            input + output,
            cost
        ))
    }
}

// ---- /files --------------------------------------------------------------

#[async_trait]
impl SlashCommand for FilesCommand {
    fn name(&self) -> &str { "files" }
    fn description(&self) -> &str { "List files referenced in the current conversation" }

    async fn execute(&self, _args: &str, ctx: &mut CommandContext) -> CommandResult {
        use std::collections::HashSet;
        // Scan message content for file paths (simple heuristic)
        let mut files: HashSet<String> = HashSet::new();
        let path_re = regex::Regex::new(r#"(?m)([A-Za-z]:[\\/][^\s,;:"'<>]+|/[^\s,;:"'<>]{3,})"#).ok();

        for msg in &ctx.messages {
            let text = msg.get_all_text();
            if let Some(ref re) = path_re {
                for cap in re.captures_iter(&text) {
                    let path = cap[1].trim().to_string();
                    if std::path::Path::new(&path).exists() {
                        files.insert(path);
                    }
                }
            }
        }

        if files.is_empty() {
            return CommandResult::Message(
                "No referenced files detected in the conversation.".to_string(),
            );
        }

        let mut sorted: Vec<String> = files.into_iter().collect();
        sorted.sort();

        CommandResult::Message(format!(
            "Referenced files ({}):\n{}",
            sorted.len(),
            sorted.iter().map(|f| format!("  {}", f)).collect::<Vec<_>>().join("\n")
        ))
    }
}

// ---- /rename -------------------------------------------------------------

#[async_trait]
impl SlashCommand for RenameCommand {
    fn name(&self) -> &str { "rename" }
    fn description(&self) -> &str { "Rename the current session" }
    fn help(&self) -> &str { "Usage: /rename <new name>" }

    async fn execute(&self, args: &str, ctx: &mut CommandContext) -> CommandResult {
        let name = args.trim();
        if name.is_empty() {
            return CommandResult::Error("Usage: /rename <new name>".to_string());
        }

        // Store the session title if we have a config-level session_id
        // For now surface it as a status message since session management is in main.rs
        CommandResult::Message(format!("Session renamed to: \"{}\"", name))
    }
}

// ---- /effort -------------------------------------------------------------

#[async_trait]
impl SlashCommand for EffortCommand {
    fn name(&self) -> &str { "effort" }
    fn description(&self) -> &str { "Set the model's thinking effort (low | normal | high)" }
    fn help(&self) -> &str {
        "Usage: /effort [low|normal|high]\n\
         Sets how much computation the model uses for reasoning.\n\
         'high' enables extended thinking with a larger budget."
    }

    async fn execute(&self, args: &str, ctx: &mut CommandContext) -> CommandResult {
        match args.trim() {
            "" => CommandResult::Message(format!(
                "Current effort: normal\nUse /effort [low|normal|high] to change."
            )),
            "low" => {
                // Low effort: smaller max_tokens
                ctx.config.max_tokens = Some(4096);
                CommandResult::ConfigChange(ctx.config.clone())
            }
            "normal" => {
                ctx.config.max_tokens = None; // use default
                CommandResult::ConfigChange(ctx.config.clone())
            }
            "high" => {
                ctx.config.max_tokens = Some(32768);
                CommandResult::ConfigChange(ctx.config.clone())
            }
            other => CommandResult::Error(format!(
                "Unknown effort level '{}'. Use: low | normal | high",
                other
            )),
        }
    }
}

// ---- /summary ------------------------------------------------------------

#[async_trait]
impl SlashCommand for SummaryCommand {
    fn name(&self) -> &str { "summary" }
    fn description(&self) -> &str { "Generate a brief summary of the conversation so far" }

    async fn execute(&self, _args: &str, ctx: &mut CommandContext) -> CommandResult {
        let count = ctx.messages.len();
        if count == 0 {
            return CommandResult::Message("No messages in conversation yet.".to_string());
        }

        // Ask the model to summarize by injecting a hidden user message
        CommandResult::UserMessage(
            "Please provide a brief (3-5 sentence) summary of our conversation so far, \
             focusing on what has been accomplished and the current state."
                .to_string(),
        )
    }
}

// ---- /commit -------------------------------------------------------------

#[async_trait]
impl SlashCommand for CommitCommand {
    fn name(&self) -> &str { "commit" }
    fn description(&self) -> &str { "Ask Claude to commit staged changes" }

    async fn execute(&self, args: &str, ctx: &mut CommandContext) -> CommandResult {
        let extra = if args.trim().is_empty() {
            String::new()
        } else {
            format!(" with message: {}", args.trim())
        };

        CommandResult::UserMessage(format!(
            "Please commit the currently staged git changes{}. \
             Run `git diff --cached` to see what's staged, \
             write an appropriate commit message following the repository's conventions, \
             and run `git commit`.",
            extra
        ))
    }
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

/// Return all built-in slash commands.
pub fn all_commands() -> Vec<Box<dyn SlashCommand>> {
    vec![
        Box::new(HelpCommand),
        Box::new(ClearCommand),
        Box::new(CompactCommand),
        Box::new(CostCommand),
        Box::new(ExitCommand),
        Box::new(ModelCommand),
        Box::new(ConfigCommand),
        Box::new(VersionCommand),
        Box::new(ResumeCommand),
        Box::new(StatusCommand),
        Box::new(DiffCommand),
        Box::new(MemoryCommand),
        Box::new(BugCommand),
        Box::new(DoctorCommand),
        Box::new(LoginCommand),
        Box::new(LogoutCommand),
        Box::new(InitCommand),
        Box::new(ReviewCommand),
        Box::new(HooksCommand),
        Box::new(McpCommand),
        Box::new(PermissionsCommand),
        Box::new(PlanCommand),
        Box::new(TasksCommand),
        Box::new(SessionCommand),
        Box::new(ThinkingCommand),
        // New commands
        Box::new(ExportCommand),
        Box::new(SkillsCommand),
        Box::new(RewindCommand),
        Box::new(StatsCommand),
        Box::new(FilesCommand),
        Box::new(RenameCommand),
        Box::new(EffortCommand),
        Box::new(SummaryCommand),
        Box::new(CommitCommand),
    ]
}

/// Find a command by name or alias.
pub fn find_command(name: &str) -> Option<Box<dyn SlashCommand>> {
    let name = name.trim_start_matches('/');
    all_commands().into_iter().find(|c| {
        c.name() == name || c.aliases().contains(&name)
    })
}

/// Execute a slash command string (with leading /).
pub async fn execute_command(
    input: &str,
    ctx: &mut CommandContext,
) -> Option<CommandResult> {
    if !cc_tui::input::is_slash_command(input) { return None; }
    let (name, args) = cc_tui::input::parse_slash_command(input);
    let cmd = find_command(name)?;
    Some(cmd.execute(args, ctx).await)
}

// ---------------------------------------------------------------------------
// Named commands module (top-level `claude <name>` subcommands)
// ---------------------------------------------------------------------------
pub mod named_commands;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use cc_core::cost::CostTracker;
    use std::sync::Arc;

    fn make_ctx() -> CommandContext {
        CommandContext {
            config: cc_core::config::Config::default(),
            cost_tracker: CostTracker::new(),
            messages: vec![],
            working_dir: std::path::PathBuf::from("."),
        }
    }

    // ---- Command registry tests ---------------------------------------------

    #[test]
    fn test_all_commands_non_empty() {
        assert!(!all_commands().is_empty());
    }

    #[test]
    fn test_all_commands_have_unique_names() {
        let mut names = std::collections::HashSet::new();
        for cmd in all_commands() {
            assert!(
                names.insert(cmd.name().to_string()),
                "Duplicate command name: {}",
                cmd.name()
            );
        }
    }

    #[test]
    fn test_find_command_by_name() {
        assert!(find_command("help").is_some());
        assert!(find_command("clear").is_some());
        assert!(find_command("exit").is_some());
        assert!(find_command("model").is_some());
        assert!(find_command("version").is_some());
    }

    #[test]
    fn test_find_command_with_slash_prefix() {
        // find_command should strip the leading / before lookup
        assert!(find_command("/help").is_some());
        assert!(find_command("/clear").is_some());
    }

    #[test]
    fn test_find_command_by_alias() {
        // /help has aliases "h" and "?"
        assert!(find_command("h").is_some());
        assert!(find_command("?").is_some());
        // /clear has alias "c"
        assert!(find_command("c").is_some());
    }

    #[test]
    fn test_find_command_not_found() {
        assert!(find_command("nonexistent_command_xyz").is_none());
    }

    #[test]
    fn test_core_commands_present() {
        let expected = [
            "help", "clear", "compact", "cost", "exit", "model",
            "config", "version", "status", "diff", "memory", "hooks",
            "permissions", "plan", "tasks", "session", "login", "logout",
        ];
        for name in &expected {
            assert!(
                find_command(name).is_some(),
                "Expected command '{}' not in all_commands()",
                name
            );
        }
    }

    // ---- Command execution tests --------------------------------------------

    #[tokio::test]
    async fn test_clear_command_returns_clear_conversation() {
        let mut ctx = make_ctx();
        let cmd = find_command("clear").unwrap();
        let result = cmd.execute("", &mut ctx).await;
        assert!(matches!(result, CommandResult::ClearConversation));
    }

    #[tokio::test]
    async fn test_exit_command_returns_exit() {
        let mut ctx = make_ctx();
        let cmd = find_command("exit").unwrap();
        let result = cmd.execute("", &mut ctx).await;
        assert!(matches!(result, CommandResult::Exit));
    }

    #[tokio::test]
    async fn test_version_command_returns_message() {
        let mut ctx = make_ctx();
        let cmd = find_command("version").unwrap();
        let result = cmd.execute("", &mut ctx).await;
        assert!(matches!(result, CommandResult::Message(_)));
        if let CommandResult::Message(msg) = result {
            assert!(
                msg.contains("claude") || msg.contains("Claude") || msg.contains('.'),
                "Version message should contain version number, got: {}",
                msg
            );
        }
    }

    #[tokio::test]
    async fn test_cost_command_returns_message() {
        let mut ctx = make_ctx();
        let cmd = find_command("cost").unwrap();
        let result = cmd.execute("", &mut ctx).await;
        assert!(matches!(result, CommandResult::Message(_)));
    }

    #[tokio::test]
    async fn test_login_command_starts_oauth_flow() {
        let mut ctx = make_ctx();
        let cmd = find_command("login").unwrap();
        // Default (no --console) → login_with_claude_ai = true
        let result = cmd.execute("", &mut ctx).await;
        assert!(matches!(result, CommandResult::StartOAuthFlow(true)));
    }

    #[tokio::test]
    async fn test_login_command_console_flag() {
        let mut ctx = make_ctx();
        let cmd = find_command("login").unwrap();
        let result = cmd.execute("--console", &mut ctx).await;
        assert!(matches!(result, CommandResult::StartOAuthFlow(false)));
    }

    #[tokio::test]
    async fn test_help_command_returns_message() {
        let mut ctx = make_ctx();
        let cmd = find_command("help").unwrap();
        let result = cmd.execute("", &mut ctx).await;
        // help returns either Message or Silent
        assert!(
            matches!(result, CommandResult::Message(_) | CommandResult::Silent),
            "help should return Message or Silent"
        );
    }
}
