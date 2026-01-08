//! Help system for Oxur REPL
//!
//! Provides beautiful tiered help with:
//! - Overview via (help)
//! - Detailed topics via (help <topic>)
//! - Color-formatted output

use std::collections::HashMap;

/// Help system for the Oxur REPL
///
/// Provides tiered help documentation with beautiful formatting.
pub struct HelpSystem {
    topics: HashMap<String, String>,
    color_enabled: bool,
}

impl HelpSystem {
    /// Create a new help system
    ///
    /// # Arguments
    ///
    /// * `color_enabled` - Whether to use ANSI color codes in output
    pub fn new(color_enabled: bool) -> Self {
        let mut topics = HashMap::new();

        // Initialize all topics
        topics.insert("basics".to_string(), Self::generate_basics_topic(color_enabled));
        topics.insert("evaluation".to_string(), Self::generate_evaluation_topic(color_enabled));
        topics.insert("keyboard".to_string(), Self::generate_keyboard_topic(color_enabled));
        topics.insert("sessions".to_string(), Self::generate_sessions_topic(color_enabled));
        topics.insert("commands".to_string(), Self::generate_commands_topic(color_enabled));
        topics.insert("modes".to_string(), Self::generate_modes_topic(color_enabled));
        topics.insert("performance".to_string(), Self::generate_performance_topic(color_enabled));
        topics.insert("stats".to_string(), Self::generate_stats_topic(color_enabled));

        Self { topics, color_enabled }
    }

    /// Show the main help overview
    pub fn show_overview(&self) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&self.header("Oxur REPL Help"));
        output.push('\n');

        // Getting Started
        output.push_str(&self.section("GETTING STARTED"));
        output.push_str("Type any Lisp expression at the prompt to evaluate it:\n");
        output.push_str(&self.example("  (+ 1 2)", Some("3")));
        output.push_str(&self.example("  (* 6 7)", Some("42")));
        output.push_str(&self.example("  (println! \"Hi\")", Some("prints and returns ()")));
        output.push('\n');

        // Control Commands
        output.push_str(&self.section("CONTROL COMMANDS"));
        output.push_str(&self.command("(quit), (q), (exit)", "Exit the REPL"));
        output.push_str(&self.command("(help)", "Show this help overview"));
        output.push_str(&self.command("(help <topic>)", "Show detailed help for a topic"));
        output.push_str(&self.command("(stats)", "Show session statistics"));
        output.push_str(&self.command("(stats <view>)", "Show specific stats view"));
        output.push_str(&self.command("Ctrl-D", "Exit the REPL (EOF)"));
        output.push_str(&self.command("Ctrl-C", "Cancel current input line"));
        output.push('\n');

        // Keyboard Shortcuts
        output.push_str(&self.section("KEYBOARD SHORTCUTS"));
        output.push_str(&self.command("↑ / ↓", "Navigate command history"));
        output.push_str(&self.command("Ctrl-A", "Move to start of line"));
        output.push_str(&self.command("Ctrl-E", "Move to end of line"));
        output.push_str(&self.command("Ctrl-K", "Kill text from cursor to end"));
        output.push_str(&self.command("Ctrl-U", "Kill text from start to cursor"));
        output.push_str(&self.command("Ctrl-R", "Search history (incremental)"));
        output.push('\n');

        // Evaluation
        output.push_str(&self.section("EVALUATION"));
        output.push_str("Oxur uses three-tier execution for optimal performance:\n");
        output.push_str(&self.command("Tier 1: Calculator", "(~1ms)     - Simple arithmetic"));
        output.push_str(&self.command("Tier 2: Cached JIT", "(~1-5ms)   - Previously compiled"));
        output.push_str(&self.command("Tier 3: Fresh JIT", "(~50-300ms) - New compilation"));
        output.push('\n');
        output.push_str(&self.note("⚡ TIP: Repeated evaluations are cached for speed!"));
        output.push('\n');

        // Help Topics
        output.push_str(&self.section("HELP TOPICS"));
        output.push_str(&self.command("basics", "Getting started with Oxur"));
        output.push_str(&self.command("evaluation", "How evaluation works in detail"));
        output.push_str(&self.command("keyboard", "All keyboard shortcuts and editing"));
        output.push_str(&self.command("sessions", "Session management and modes"));
        output.push_str(&self.command("commands", "All special commands reference"));
        output.push_str(&self.command("modes", "Lisp vs Sexpr evaluation modes"));
        output.push_str(&self.command("performance", "Understanding execution tiers"));
        output.push_str(&self.command("stats", "Statistics and instrumentation"));
        output.push('\n');
        output.push_str("Type ");
        output.push_str(&self.cmd_text("(help <topic>)"));
        output.push_str(" for detailed information on any topic.\n");
        output.push('\n');
        output.push_str("For more information, visit: ");
        output.push_str(&self.cmd_text("http://oxur.li/"));
        output.push('\n');

        output
    }

    /// Show help for a specific topic
    ///
    /// Returns None if the topic doesn't exist
    pub fn show_topic(&self, topic: &str) -> Option<String> {
        self.topics.get(topic).cloned()
    }

    /// List all available help topics
    #[allow(dead_code)]
    pub fn list_topics(&self) -> Vec<&str> {
        let mut topics: Vec<&str> = self.topics.keys().map(|s| s.as_str()).collect();
        topics.sort_unstable();
        topics
    }

    // ========================================================================
    // Formatting Helpers
    // ========================================================================

    /// Format a header with box drawing
    fn header(&self, text: &str) -> String {
        let width = text.len() + 4;
        let top = format!("╭─{}─╮\n", "─".repeat(width - 2));
        let middle = format!("│  {}{}{}  │\n", self.bold_cyan(), text, self.reset());
        let bottom = format!("╰─{}─╯\n", "─".repeat(width - 2));
        format!("{}{}{}", top, middle, bottom)
    }

    /// Format a section header
    fn section(&self, title: &str) -> String {
        format!("{}{}{}\n{}\n", self.bold_cyan(), title, self.reset(), self.divider())
    }

    /// Format a command with description
    fn command(&self, cmd: &str, desc: &str) -> String {
        format!("  {}{:<20}{}  {}\n", self.yellow(), cmd, self.reset(), desc)
    }

    /// Format an example with optional result
    fn example(&self, code: &str, result: Option<&str>) -> String {
        if let Some(res) = result {
            format!("{}{}{}  → {}\n", self.green(), code, self.reset(), res)
        } else {
            format!("{}{}{}\n", self.green(), code, self.reset())
        }
    }

    /// Format command text (inline)
    fn cmd_text(&self, text: &str) -> String {
        format!("{}{}{}", self.yellow(), text, self.reset())
    }

    /// Format a divider line
    fn divider(&self) -> String {
        format!("{}{}{}\n", self.dim(), "─".repeat(60), self.reset())
    }

    /// Format a note or tip
    fn note(&self, text: &str) -> String {
        format!("{}{}{}\n", self.italic_cyan(), text, self.reset())
    }

    // ========================================================================
    // ANSI Color Codes
    // ========================================================================

    fn bold_cyan(&self) -> &str {
        if self.color_enabled {
            "\x1b[1;36m"
        } else {
            ""
        }
    }

    fn yellow(&self) -> &str {
        if self.color_enabled {
            "\x1b[1;33m"
        } else {
            ""
        }
    }

    fn green(&self) -> &str {
        if self.color_enabled {
            "\x1b[32m"
        } else {
            ""
        }
    }

    fn italic_cyan(&self) -> &str {
        if self.color_enabled {
            "\x1b[3;36m"
        } else {
            ""
        }
    }

    fn dim(&self) -> &str {
        if self.color_enabled {
            "\x1b[2;37m"
        } else {
            ""
        }
    }

    fn reset(&self) -> &str {
        if self.color_enabled {
            "\x1b[0m"
        } else {
            ""
        }
    }

    // ========================================================================
    // Topic Content Generators
    // ========================================================================

    fn generate_basics_topic(color_enabled: bool) -> String {
        let help = Self { topics: HashMap::new(), color_enabled };
        let mut output = String::new();

        output.push_str(&help.header("Getting Started with Oxur"));
        output.push('\n');

        output.push_str(&help.section("OVERVIEW"));
        output.push_str("Oxur is a Lisp dialect that compiles to Rust. The REPL provides an\n");
        output.push_str("interactive environment for evaluating Oxur expressions and seeing\n");
        output.push_str("immediate results.\n\n");

        output.push_str(&help.section("FIRST STEPS"));
        output.push_str("Start by trying simple arithmetic:\n\n");
        output.push_str(&help.example("  (+ 1 2)", Some("3")));
        output.push_str(&help.example("  (* 6 7)", Some("42")));
        output.push_str(&help.example("  (- 100 25)", Some("75")));
        output.push('\n');

        output.push_str("You can nest expressions:\n\n");
        output.push_str(&help.example("  (+ (* 2 3) (/ 10 2))", Some("11")));
        output.push('\n');

        output.push_str(&help.section("DEFINING FUNCTIONS"));
        output.push_str("Define functions with ");
        output.push_str(&help.cmd_text("defn"));
        output.push_str(":\n\n");
        output.push_str(&help.example("  (defn square [x] (* x x))", None));
        output.push_str(&help.example("  (square 5)", Some("25")));
        output.push('\n');

        output.push_str(&help.section("VARIABLES"));
        output.push_str("Bind values to names with ");
        output.push_str(&help.cmd_text("let"));
        output.push_str(":\n\n");
        output.push_str(&help.example("  (let x 42)", None));
        output.push_str(&help.example("  (+ x 8)", Some("50")));
        output.push('\n');

        output.push_str(&help.note("💡 TIP: Press ↑ to recall previous commands from history"));
        output.push('\n');

        output.push_str("See also: ");
        output.push_str(&help.cmd_text("(help evaluation)"));
        output.push_str(", ");
        output.push_str(&help.cmd_text("(help modes)"));
        output.push('\n');

        output
    }

    fn generate_evaluation_topic(color_enabled: bool) -> String {
        let help = Self { topics: HashMap::new(), color_enabled };
        let mut output = String::new();

        output.push_str(&help.header("Evaluation System"));
        output.push('\n');

        output.push_str(&help.section("OVERVIEW"));
        output
            .push_str("Oxur evaluates Lisp expressions by compiling them to Rust and executing\n");
        output.push_str(
            "the generated code. To maximize performance, it uses a three-tier system.\n\n",
        );

        output.push_str(&help.section("THREE-TIER EXECUTION"));
        output.push('\n');

        output.push_str(&help.cmd_text("Tier 1: Calculator (~1ms)"));
        output.push('\n');
        output.push_str(&help.divider());
        output.push_str("Simple arithmetic and basic operations are evaluated directly\n");
        output.push_str("without compilation. This is the fastest path.\n\n");
        output.push_str("Examples:\n");
        output.push_str(&help.example("  (+ 1 2)", Some("Direct calculation")));
        output.push_str(&help.example("  (* 10 20)", Some("Direct calculation")));
        output.push('\n');

        output.push_str(&help.cmd_text("Tier 2: Cached JIT (~1-5ms)"));
        output.push('\n');
        output.push_str(&help.divider());
        output.push_str("Previously compiled expressions are cached in memory. When you\n");
        output.push_str("evaluate the same expression again, the cached version is used.\n\n");
        output.push_str("Example:\n");
        output
            .push_str(&help.example("  (defn square [x] (* x x))", Some("Compiles once (~50ms)")));
        output.push_str(&help.example("  (square 5)", Some("Uses cache (~2ms)")));
        output.push_str(&help.example("  (square 10)", Some("Uses cache (~2ms)")));
        output.push('\n');

        output.push_str(&help.cmd_text("Tier 3: Fresh JIT (~50-300ms)"));
        output.push('\n');
        output.push_str(&help.divider());
        output.push_str("New expressions are compiled on-the-fly. The first evaluation\n");
        output.push_str("is slower but subsequent calls use the cache.\n\n");

        output.push_str(
            &help.note("⚡ TIP: The REPL shows you which tier was used for each evaluation"),
        );
        output.push('\n');

        output.push_str("See also: ");
        output.push_str(&help.cmd_text("(help performance)"));
        output.push('\n');

        output
    }

    fn generate_keyboard_topic(color_enabled: bool) -> String {
        let help = Self { topics: HashMap::new(), color_enabled };
        let mut output = String::new();

        output.push_str(&help.header("Keyboard Shortcuts"));
        output.push('\n');

        output.push_str(&help.section("NAVIGATION"));
        output.push_str(&help.command("←  →", "Move cursor left/right by character"));
        output.push_str(&help.command("Ctrl-A", "Move to start of line"));
        output.push_str(&help.command("Ctrl-E", "Move to end of line"));
        output.push_str(&help.command("Alt-B", "Move backward by word"));
        output.push_str(&help.command("Alt-F", "Move forward by word"));
        output.push('\n');

        output.push_str(&help.section("EDITING"));
        output.push_str(&help.command("Ctrl-K", "Kill text from cursor to end of line"));
        output.push_str(&help.command("Ctrl-U", "Kill text from start to cursor"));
        output.push_str(&help.command("Ctrl-W", "Kill word before cursor"));
        output.push_str(&help.command("Alt-D", "Kill word after cursor"));
        output.push_str(&help.command("Ctrl-Y", "Yank (paste) killed text"));
        output.push_str(&help.command("Ctrl-_", "Undo last edit"));
        output.push('\n');

        output.push_str(&help.section("HISTORY"));
        output.push_str(&help.command("↑  ↓", "Navigate command history"));
        output.push_str(&help.command("Ctrl-R", "Search history (incremental search)"));
        output.push_str(&help.command("Ctrl-G", "Cancel incremental search"));
        output.push('\n');

        output.push_str(&help.section("CONTROL"));
        output.push_str(&help.command("Ctrl-C", "Cancel current input line"));
        output.push_str(&help.command("Ctrl-D", "Exit REPL (if line is empty)"));
        output.push_str(&help.command("Ctrl-L", "Clear screen"));
        output.push_str(&help.command("Enter", "Submit current line for evaluation"));
        output.push('\n');

        output.push_str(&help.note("💡 TIP: History is saved between REPL sessions"));
        output.push('\n');

        output
    }

    fn generate_sessions_topic(color_enabled: bool) -> String {
        let help = Self { topics: HashMap::new(), color_enabled };
        let mut output = String::new();

        output.push_str(&help.header("Session Management"));
        output.push('\n');

        output.push_str(&help.section("OVERVIEW"));
        output.push_str("A session represents an isolated evaluation environment with its\n");
        output.push_str("own state, variables, and compilation cache. Each REPL connection\n");
        output.push_str("creates a new session automatically.\n\n");

        output.push_str(&help.section("SESSION MODES"));
        output.push_str("Sessions can operate in two modes:\n\n");

        output.push_str(&help.cmd_text("Lisp Mode (default)"));
        output.push('\n');
        output.push_str(&help.divider());
        output.push_str("Full Lisp syntax with reader macros and syntactic sugar.\n");
        output.push_str("This is the standard interactive mode.\n\n");
        output.push_str("Examples: ");
        output.push_str(&help.cmd_text("(+ 1 2)"));
        output.push_str(", ");
        output.push_str(&help.cmd_text("[1 2 3]"));
        output.push_str(", ");
        output.push_str(&help.cmd_text("{:key \"value\"}"));
        output.push_str("\n\n");

        output.push_str(&help.cmd_text("Sexpr Mode"));
        output.push('\n');
        output.push_str(&help.divider());
        output.push_str("Raw S-expressions in canonical form (what Lisp expands to).\n");
        output.push_str("Used for debugging the compiler or working with the AST directly.\n\n");
        output.push_str("Example: ");
        output.push_str(&help.cmd_text("(Add (Lit 1) (Lit 2))"));
        output.push_str("\n\n");

        output.push_str(&help.section("MULTIPLE SESSIONS"));
        output.push_str("When running in server mode, multiple clients can connect\n");
        output.push_str("simultaneously, each with their own isolated session.\n\n");

        output.push_str(&help.note("💡 TIP: Session state persists for the duration of the REPL"));
        output.push('\n');

        output.push_str("See also: ");
        output.push_str(&help.cmd_text("(help modes)"));
        output.push('\n');

        output
    }

    fn generate_commands_topic(color_enabled: bool) -> String {
        let help = Self { topics: HashMap::new(), color_enabled };
        let mut output = String::new();

        output.push_str(&help.header("Special Commands Reference"));
        output.push('\n');

        output.push_str(&help.section("REPL CONTROL"));
        output.push_str(&help.command("(quit)", "Exit the REPL gracefully"));
        output.push_str(&help.command("(q)", "Exit the REPL (short alias for quit)"));
        output.push_str(&help.command("(exit)", "Exit the REPL (alias for quit)"));
        output.push_str(&help.command("(help)", "Show help overview"));
        output.push_str(&help.command("(help <topic>)", "Show detailed help for a specific topic"));
        output.push('\n');

        output.push_str(&help.section("STATISTICS"));
        output.push_str(&help.command("(stats)", "Show session summary (tiers, cache hit rate)"));
        output
            .push_str(&help.command("(stats execution)", "Show detailed tier performance metrics"));
        output.push_str(&help.command("(stats cache)", "Show cache hit/miss statistics"));
        output.push_str(&help.command("(stats resources)", "Show memory, disk, and file usage"));
        output.push_str(&help.command("(stats server)", "Show server metrics (connect mode only)"));
        output.push_str(&help.command("(stats subprocess)", "Show subprocess lifecycle stats"));
        output.push('\n');

        output.push_str(&help.section("KEYBOARD SHORTCUTS"));
        output.push_str(&help.command("Ctrl-D", "Exit REPL if line is empty"));
        output.push_str(&help.command("Ctrl-C", "Cancel current input and start fresh"));
        output.push_str(&help.command("Ctrl-L", "Clear the screen"));
        output.push('\n');

        output.push_str(&help.section("EVALUATION"));
        output.push_str("Any expression not matching a special command is evaluated\n");
        output.push_str("as Oxur code using the current session's evaluation mode.\n\n");

        output
            .push_str(&help.note(
                "💡 TIP: Special commands are handled locally and don't require compilation",
            ));
        output.push('\n');

        output.push_str("See also: ");
        output.push_str(&help.cmd_text("(help keyboard)"));
        output.push_str(", ");
        output.push_str(&help.cmd_text("(help evaluation)"));
        output.push_str(", ");
        output.push_str(&help.cmd_text("(help stats)"));
        output.push('\n');

        output
    }

    fn generate_modes_topic(color_enabled: bool) -> String {
        let help = Self { topics: HashMap::new(), color_enabled };
        let mut output = String::new();

        output.push_str(&help.header("Evaluation Modes"));
        output.push('\n');

        output.push_str(&help.section("OVERVIEW"));
        output.push_str("Oxur supports two evaluation modes: Lisp (user-friendly) and\n");
        output.push_str("Sexpr (canonical form). The mode is set when starting the REPL.\n\n");

        output.push_str(&help.section("LISP MODE (DEFAULT)"));
        output.push_str("Full Lisp syntax with reader macros, syntactic sugar, and\n");
        output.push_str("familiar Lisp idioms. This is the recommended mode for\n");
        output.push_str("interactive development.\n\n");

        output.push_str("Features:\n");
        output.push_str("  • Reader macros: ");
        output.push_str(&help.cmd_text("'expr"));
        output.push_str(" → ");
        output.push_str(&help.cmd_text("(quote expr)"));
        output.push('\n');
        output.push_str("  • Vector literals: ");
        output.push_str(&help.cmd_text("[1 2 3]"));
        output.push('\n');
        output.push_str("  • Map literals: ");
        output.push_str(&help.cmd_text("{:key \"value\"}"));
        output.push('\n');
        output.push_str("  • Set literals: ");
        output.push_str(&help.cmd_text("#{1 2 3}"));
        output.push_str("\n\n");

        output.push_str(&help.section("SEXPR MODE"));
        output.push_str("Raw S-expressions in canonical form. This mode shows you\n");
        output.push_str("exactly what the Lisp reader produces after expansion.\n\n");

        output.push_str("Use cases:\n");
        output.push_str("  • Debugging macro expansions\n");
        output.push_str("  • Understanding compiler internals\n");
        output.push_str("  • Working directly with the AST\n");
        output.push_str("  • Testing parser/expander behavior\n\n");

        output.push_str(&help.section("SWITCHING MODES"));
        output.push_str("The mode is set when launching the REPL:\n\n");
        output.push_str(&help.example("  oxur repl", Some("Starts in Lisp mode")));
        output.push_str(&help.example("  oxur repl --mode sexpr", Some("Starts in Sexpr mode")));
        output.push('\n');

        output.push_str(&help.note("💡 TIP: Most users should use Lisp mode for interactive work"));
        output.push('\n');

        output.push_str("See also: ");
        output.push_str(&help.cmd_text("(help sessions)"));
        output.push('\n');

        output
    }

    fn generate_performance_topic(color_enabled: bool) -> String {
        let help = Self { topics: HashMap::new(), color_enabled };
        let mut output = String::new();

        output.push_str(&help.header("Performance and Optimization"));
        output.push('\n');

        output.push_str(&help.section("EXECUTION TIERS"));
        output.push_str("Oxur uses a three-tier execution strategy to balance\n");
        output.push_str("startup time, throughput, and interactive responsiveness.\n\n");

        output.push_str(&help.cmd_text("Tier 1: Calculator (~1ms)"));
        output.push('\n');
        output.push_str(&help.divider());
        output.push_str("Eligible expressions:\n");
        output.push_str("  • Pure arithmetic: (+, -, *, /)\n");
        output.push_str("  • Literal values only (no variables)\n");
        output.push_str("  • Nested arithmetic operations\n\n");
        output.push_str("Performance: Sub-millisecond, no compilation overhead\n\n");

        output.push_str(&help.cmd_text("Tier 2: Cached JIT (~1-5ms)"));
        output.push('\n');
        output.push_str(&help.divider());
        output.push_str("Caching strategy:\n");
        output.push_str("  • Content-addressed: Same code → same cache key\n");
        output.push_str("  • Session-local: Each session has its own cache\n");
        output.push_str("  • Persistent: Cache survives across evaluations\n\n");
        output.push_str("Performance: Near-instant for cached code\n\n");

        output.push_str(&help.cmd_text("Tier 3: Fresh JIT (~50-300ms)"));
        output.push('\n');
        output.push_str(&help.divider());
        output.push_str("Compilation pipeline:\n");
        output.push_str("  1. Parse Oxur → AST\n");
        output.push_str("  2. Expand macros → Core forms\n");
        output.push_str("  3. Lower → Rust AST\n");
        output.push_str("  4. Codegen → Rust source\n");
        output.push_str("  5. Compile → Dynamic library\n");
        output.push_str("  6. Load → Execute\n\n");
        output.push_str("Performance: One-time cost, then cached\n\n");

        output.push_str(&help.section("OPTIMIZATION TIPS"));
        output.push_str("⚡ Reuse code: Repeated evaluations use cached compilation\n");
        output.push_str("⚡ Simple math: Use calculator tier when possible\n");
        output.push_str("⚡ Define functions: Compiled once, called many times\n");
        output.push_str("⚡ Batch work: Define multiple functions, then use them\n\n");

        output.push_str(
            &help.note("💡 TIP: Watch the execution tier in output to understand performance"),
        );
        output.push('\n');

        output.push_str("See also: ");
        output.push_str(&help.cmd_text("(help evaluation)"));
        output.push('\n');

        output
    }

    fn generate_stats_topic(color_enabled: bool) -> String {
        let help = Self { topics: HashMap::new(), color_enabled };
        let mut output = String::new();

        output.push_str(&help.header("Statistics and Instrumentation"));
        output.push('\n');

        output.push_str(&help.section("OVERVIEW"));
        output.push_str("The REPL provides comprehensive statistics and instrumentation to help\n");
        output.push_str("you understand performance, resource usage, and execution patterns.\n");
        output.push_str("All statistics use styled tables for clear presentation.\n\n");

        output.push_str(&help.section("AVAILABLE VIEWS"));
        output.push('\n');

        output.push_str(&help.cmd_text("(stats)"));
        output.push_str(" - Session Summary\n");
        output.push_str(&help.divider());
        output.push_str("Shows a high-level overview of your REPL session:\n");
        output.push_str("  • Total evaluations performed\n");
        output.push_str("  • Cache hit rate (hits, misses, percentage)\n");
        output.push_str("  • Execution tier summary with percentiles (p50, p95, p99)\n\n");
        output.push_str("Use this view to get a quick snapshot of session performance.\n\n");

        output.push_str(&help.cmd_text("(stats execution)"));
        output.push_str(" - Detailed Tier Breakdown\n");
        output.push_str(&help.divider());
        output.push_str("Shows detailed performance metrics for each execution tier:\n");
        output.push_str("  • Count: Number of evaluations at this tier\n");
        output.push_str("  • Min: Fastest evaluation time\n");
        output.push_str("  • p50 (median): Typical evaluation time\n");
        output.push_str("  • p95: 95th percentile (outliers excluded)\n");
        output.push_str("  • p99: 99th percentile (almost all samples)\n");
        output.push_str("  • Max: Slowest evaluation time\n\n");
        output.push_str("Use this view to understand tier-specific performance patterns.\n\n");

        output.push_str(&help.cmd_text("(stats cache)"));
        output.push_str(" - Cache Metrics\n");
        output.push_str(&help.divider());
        output.push_str("Shows caching effectiveness:\n");
        output.push_str("  • Hits: Number of cache hits (fast path)\n");
        output.push_str("  • Misses: Number of cache misses (compilation required)\n");
        output.push_str("  • Hit Rate: Percentage of evaluations served from cache\n\n");
        output.push_str("A high hit rate (>70%) indicates good code reuse patterns.\n\n");

        output.push_str(&help.cmd_text("(stats resources)"));
        output.push_str(" - Resource Usage\n");
        output.push_str(&help.divider());
        output.push_str("Shows system resource consumption:\n\n");
        output.push_str("Memory:\n");
        output.push_str("  • Process RSS: Resident memory (actual RAM used)\n");
        output.push_str("  • Virtual Memory: Total address space\n");
        output.push_str("  • Process ID: Operating system PID\n\n");
        output.push_str("Session Directory:\n");
        output.push_str("  • Location: Path to session temp directory\n");
        output.push_str("  • Files: Count of compiled artifacts\n");
        output.push_str("  • Disk Usage: Total space consumed\n\n");
        output.push_str("Artifact Cache (Global):\n");
        output.push_str("  • Entries: Number of cached artifacts\n");
        output.push_str("  • Total Size: Disk space used by cache\n");
        output.push_str("  • Oldest Entry: Age of oldest cached item\n");
        output.push_str("  • Cache Directory: Path to global cache\n\n");

        output.push_str(&help.cmd_text("(stats server)"));
        output.push_str(" - Server Metrics (Connect Mode Only)\n");
        output.push_str(&help.divider());
        output.push_str("Shows server-wide metrics when connected to a remote server:\n\n");
        output.push_str("Connections:\n");
        output.push_str("  • Total Connections: All-time connection count\n");
        output.push_str("  • Active Connections: Currently connected clients\n\n");
        output.push_str("Sessions:\n");
        output.push_str("  • Total Sessions: All-time session count\n");
        output.push_str("  • Active Sessions: Currently running sessions\n\n");
        output.push_str("Requests & Responses:\n");
        output.push_str("  • Total Requests/Responses: Message counts\n");
        output.push_str("  • Successful/Errors: Response status breakdown\n");
        output.push_str("  • Success Rate: Percentage of successful operations\n\n");
        output.push_str("Note: Only available in 'connect' mode (remote server).\n");
        output.push_str("In interactive mode, use 'oxur repl serve' to run a server.\n\n");

        output.push_str(&help.cmd_text("(stats subprocess)"));
        output.push_str(" - Subprocess Lifecycle\n");
        output.push_str(&help.divider());
        output.push_str("Shows subprocess lifecycle metrics (for forked execution):\n\n");
        output.push_str("Status:\n");
        output.push_str("  • Status: Running or Stopped\n");
        output.push_str("  • Uptime: How long the subprocess has been running\n");
        output.push_str("  • Restart Count: Number of restarts since session start\n");
        output.push_str("  • Last Restart Reason: Why the subprocess last restarted\n\n");
        output.push_str("Restart reasons include:\n");
        output.push_str("  • Clean shutdown: Normal exit (code 0)\n");
        output.push_str("  • Error exit: Non-zero exit code\n");
        output.push_str("  • Segfault: Memory access violation (SIGSEGV)\n");
        output.push_str("  • Killed: Process killed (SIGKILL, possibly OOM)\n");
        output.push_str("  • Aborted: Assertion failure (SIGABRT)\n");
        output.push_str("  • User requested: Manual restart command\n\n");

        output.push_str(&help.section("PERCENTILES EXPLAINED"));
        output.push_str("Percentiles give you a better understanding of typical performance\n");
        output.push_str("than simple averages:\n\n");
        output.push_str("  • ");
        output.push_str(&help.cmd_text("p50 (median)"));
        output.push_str(": Half of evaluations are faster\n");
        output.push_str("  • ");
        output.push_str(&help.cmd_text("p95"));
        output.push_str(": 95% of evaluations are faster (excludes slow outliers)\n");
        output.push_str("  • ");
        output.push_str(&help.cmd_text("p99"));
        output.push_str(": 99% of evaluations are faster (includes most samples)\n\n");
        output.push_str("If p50 and p95 are close, performance is consistent.\n");
        output
            .push_str("If p99 is much higher than p95, you have occasional slow evaluations.\n\n");

        output.push_str(&help.section("UNDERSTANDING THE TIERS"));
        output.push_str("The stats show performance across three execution tiers:\n\n");
        output.push_str(&help.cmd_text("Calculator"));
        output.push_str(": Simple arithmetic (~1ms)\n");
        output.push_str("  Fast, no compilation, direct evaluation\n\n");
        output.push_str(&help.cmd_text("Cached"));
        output.push_str(": Previously compiled (~1-5ms)\n");
        output.push_str("  Near-instant, uses cached compilation\n\n");
        output.push_str(&help.cmd_text("JIT"));
        output.push_str(": Fresh compilation (~50-300ms)\n");
        output.push_str("  Slower first time, then cached\n\n");

        output.push_str(&help.section("OPTIMIZATION TIPS"));
        output.push_str("Use stats to guide optimization:\n\n");
        output.push_str("⚡ Low cache hit rate? Reuse more code patterns\n");
        output.push_str("⚡ High p99 on Calculator tier? Simplify expressions\n");
        output.push_str("⚡ Many JIT evaluations? Define functions for reuse\n");
        output.push_str("⚡ High memory usage? Consider restarting REPL session\n\n");

        output.push_str(&help.note("💡 TIP: Track stats over time to see performance trends"));
        output.push('\n');

        output.push_str("See also: ");
        output.push_str(&help.cmd_text("(help performance)"));
        output.push_str(", ");
        output.push_str(&help.cmd_text("(help evaluation)"));
        output.push('\n');

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_system_creation() {
        let help = HelpSystem::new(true);
        assert_eq!(help.list_topics().len(), 8);
    }

    #[test]
    fn test_overview_has_all_sections() {
        let help = HelpSystem::new(false);
        let overview = help.show_overview();
        assert!(overview.contains("GETTING STARTED"));
        assert!(overview.contains("CONTROL COMMANDS"));
        assert!(overview.contains("KEYBOARD SHORTCUTS"));
        assert!(overview.contains("EVALUATION"));
        assert!(overview.contains("HELP TOPICS"));
    }

    #[test]
    fn test_all_topics_available() {
        let help = HelpSystem::new(false);
        let topics = vec![
            "basics",
            "evaluation",
            "keyboard",
            "sessions",
            "commands",
            "modes",
            "performance",
            "stats",
        ];
        for topic in topics {
            assert!(help.show_topic(topic).is_some(), "Topic {} should exist", topic);
        }
    }

    #[test]
    fn test_color_disabled() {
        let help = HelpSystem::new(false);
        let overview = help.show_overview();
        assert!(!overview.contains("\x1b["), "Should not contain ANSI codes");
    }

    #[test]
    fn test_color_enabled() {
        let help = HelpSystem::new(true);
        let overview = help.show_overview();
        assert!(overview.contains("\x1b["), "Should contain ANSI codes");
    }

    #[test]
    fn test_unknown_topic() {
        let help = HelpSystem::new(false);
        assert!(help.show_topic("nonexistent").is_none());
    }

    #[test]
    fn test_topics_list_is_sorted() {
        let help = HelpSystem::new(false);
        let topics = help.list_topics();
        let mut sorted_topics = topics.clone();
        sorted_topics.sort_unstable();
        assert_eq!(topics, sorted_topics);
    }

    #[test]
    fn test_each_topic_has_header() {
        let help = HelpSystem::new(false);
        for topic in help.list_topics() {
            let content = help.show_topic(topic).unwrap();
            assert!(content.contains("╭─"), "Topic {} should have header", topic);
            assert!(content.contains("╰─"), "Topic {} should have header", topic);
        }
    }

    #[test]
    fn test_overview_mentions_all_topics() {
        let help = HelpSystem::new(false);
        let overview = help.show_overview();
        for topic in help.list_topics() {
            assert!(overview.contains(topic), "Overview should mention topic {}", topic);
        }
    }
}
