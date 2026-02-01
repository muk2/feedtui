mod app;
mod config;
mod creature;
mod event;
mod feeds;
mod ui;

use feeds::youtube_oauth::YouTubeOAuth;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "feedtui")]
#[command(about = "A configurable terminal dashboard for stocks, news, sports, and social feeds")]
#[command(version)]
struct Args {
    /// Path to config file
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,

    /// Refresh interval in seconds (overrides config)
    #[arg(short, long)]
    refresh: Option<u64>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize configuration with interactive wizard
    Init {
        /// Force overwrite existing config
        #[arg(short, long)]
        force: bool,
    },
    /// Show current configuration path and status
    Config,
    /// Install the binary to cargo bin directory
    Install,
    /// Authenticate with YouTube for personalized feeds
    YoutubeAuth {
        /// OAuth client ID (from Google Cloud Console)
        #[arg(long)]
        client_id: String,
        /// OAuth client secret (from Google Cloud Console)
        #[arg(long)]
        client_secret: String,
    },
    /// Clear YouTube authentication tokens
    YoutubeLogout,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Handle subcommands
    if let Some(command) = args.command {
        match command {
            Commands::Init { force } => {
                return init_config(force);
            }
            Commands::Config => {
                return show_config_info();
            }
            Commands::Install => {
                return show_install_instructions();
            }
            Commands::YoutubeAuth {
                client_id,
                client_secret,
            } => {
                return youtube_auth(client_id, client_secret).await;
            }
            Commands::YoutubeLogout => {
                return youtube_logout();
            }
        }
    }

    // Load config from ~/.feedtui/config.toml (cross-platform)
    let config_path = args.config.unwrap_or_else(|| {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".feedtui")
            .join("config.toml")
    });

    let mut config = config::Config::load(&config_path).unwrap_or_else(|e| {
        eprintln!(
            "Warning: Could not load config from {:?}: {}",
            config_path, e
        );
        eprintln!("Using default configuration...");
        eprintln!("Tip: Run 'feedtui init' to create a configuration file.\n");
        config::Config::default()
    });

    // Apply CLI overrides
    if let Some(refresh) = args.refresh {
        config.general.refresh_interval_secs = refresh;
    }

    // Run the app
    let mut app = app::App::new(config);
    app.run().await
}

fn init_config(force: bool) -> Result<()> {
    use std::io::{self, Write};

    let config_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".feedtui");
    let config_path = config_dir.join("config.toml");

    // Check if config already exists
    if config_path.exists() && !force {
        eprintln!("Config file already exists at: {:?}", config_path);
        eprintln!("Use --force to overwrite it.");
        eprintln!("\nTo edit your config manually: {}", config_path.display());
        return Ok(());
    }

    println!("=== feedtui Configuration Wizard ===\n");

    // Create config directory if it doesn't exist
    std::fs::create_dir_all(&config_dir)?;

    // Prompt for refresh interval
    print!("Refresh interval in seconds [60]: ");
    io::stdout().flush()?;
    let mut refresh_input = String::new();
    io::stdin().read_line(&mut refresh_input)?;
    let refresh_interval = refresh_input.trim().parse::<u64>().unwrap_or(60);

    // Prompt for theme
    print!("Theme (dark/light) [dark]: ");
    io::stdout().flush()?;
    let mut theme_input = String::new();
    io::stdin().read_line(&mut theme_input)?;
    let theme = theme_input.trim();
    let theme = if theme.is_empty() { "dark" } else { theme };

    // Ask about widgets
    println!("\n=== Widget Configuration ===");
    println!("Which widgets would you like to enable?\n");

    let enable_creature = prompt_yes_no("Enable Tui creature companion?", true)?;
    let enable_hackernews = prompt_yes_no("Enable Hacker News?", true)?;
    let enable_stocks = prompt_yes_no("Enable stock ticker?", true)?;
    let enable_rss = prompt_yes_no("Enable RSS feeds?", true)?;
    let enable_sports = prompt_yes_no("Enable sports scores?", false)?;
    let enable_github = prompt_yes_no("Enable GitHub dashboard?", false)?;

    // Build config content
    let mut config_content = format!(
        "[general]\nrefresh_interval_secs = {}\ntheme = \"{}\"\n\n",
        refresh_interval, theme
    );

    let mut row = 0;
    let mut col = 0;

    if enable_creature {
        config_content.push_str(&format!(
            "[[widgets]]\ntype = \"creature\"\ntitle = \"Tui\"\nshow_on_startup = true\nposition = {{ row = {}, col = {} }}\n\n",
            row, col
        ));
        col += 1;
    }

    if enable_hackernews {
        config_content.push_str(&format!(
            "[[widgets]]\ntype = \"hackernews\"\ntitle = \"Hacker News\"\nstory_count = 10\nstory_type = \"top\"\nposition = {{ row = {}, col = {} }}\n\n",
            row, col
        ));
        col += 1;
    }

    if enable_stocks {
        print!("\nEnter stock symbols (comma-separated) [AAPL,GOOGL,MSFT]: ");
        io::stdout().flush()?;
        let mut stocks_input = String::new();
        io::stdin().read_line(&mut stocks_input)?;
        let stocks = stocks_input.trim();
        let stocks = if stocks.is_empty() {
            "AAPL\", \"GOOGL\", \"MSFT"
        } else {
            stocks
        };
        let stocks_array = stocks
            .split(',')
            .map(|s| format!("\"{}\"", s.trim()))
            .collect::<Vec<_>>()
            .join(", ");

        if col >= 3 {
            row += 1;
            col = 0;
        }
        config_content.push_str(&format!(
            "[[widgets]]\ntype = \"stocks\"\ntitle = \"Portfolio\"\nsymbols = [{}]\nposition = {{ row = {}, col = {} }}\n\n",
            stocks_array, row, col
        ));
        col += 1;
    }

    if enable_rss {
        if col >= 3 {
            row += 1;
            col = 0;
        }
        config_content.push_str(&format!(
            "[[widgets]]\ntype = \"rss\"\ntitle = \"Tech News\"\nfeeds = [\n  \"https://feeds.arstechnica.com/arstechnica/technology-lab\"\n]\nmax_items = 10\nposition = {{ row = {}, col = {} }}\n\n",
            row, col
        ));
        col += 1;
    }

    if enable_sports {
        if col >= 3 {
            row += 1;
            col = 0;
        }
        config_content.push_str(&format!(
            "[[widgets]]\ntype = \"sports\"\ntitle = \"Sports\"\nleagues = [\"nba\", \"nfl\"]\nposition = {{ row = {}, col = {} }}\n\n",
            row, col
        ));
        col += 1;
    }

    if enable_github {
        println!("\n=== GitHub Configuration ===");
        print!("GitHub username: ");
        io::stdout().flush()?;
        let mut github_user = String::new();
        io::stdin().read_line(&mut github_user)?;
        let github_user = github_user.trim();

        if !github_user.is_empty() {
            if col >= 3 {
                row += 1;
                col = 0;
            }
            config_content.push_str(&format!(
                "[[widgets]]\ntype = \"github\"\ntitle = \"GitHub Dashboard\"\ntoken = \"${{GITHUB_TOKEN}}\"\nusername = \"{}\"\nshow_notifications = true\nshow_pull_requests = true\nshow_commits = true\nmax_notifications = 20\nmax_pull_requests = 10\nmax_commits = 10\nposition = {{ row = {}, col = {} }}\n\n",
                github_user, row, col
            ));
        }
    }

    // Write config file
    std::fs::write(&config_path, config_content)?;

    println!("\n✓ Configuration saved to: {}", config_path.display());
    println!("\nYou can edit this file directly or run 'feedtui init --force' to reconfigure.");
    println!("\nRun 'feedtui' to start the dashboard!");

    Ok(())
}

fn prompt_yes_no(prompt: &str, default: bool) -> Result<bool> {
    use std::io::{self, Write};

    let default_str = if default { "Y/n" } else { "y/N" };
    print!("{} [{}]: ", prompt, default_str);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    Ok(match input.as_str() {
        "" => default,
        "y" | "yes" => true,
        "n" | "no" => false,
        _ => default,
    })
}

fn show_config_info() -> Result<()> {
    let config_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".feedtui");
    let config_path = config_dir.join("config.toml");

    println!("=== feedtui Configuration ===\n");
    println!("Config directory: {}", config_dir.display());
    println!("Config file:      {}", config_path.display());

    if config_path.exists() {
        println!("Status:           ✓ Found");
        println!("\nTo edit: open {}", config_path.display());
        println!("To reconfigure: feedtui init --force");
    } else {
        println!("Status:           ✗ Not found");
        println!("\nTo create: feedtui init");
    }

    Ok(())
}

fn show_install_instructions() -> Result<()> {
    println!("=== feedtui Installation ===\n");
    println!("To install feedtui as a global command:\n");
    println!("  cargo install --path .\n");
    println!("This will install the binary to ~/.cargo/bin/feedtui");
    println!("Make sure ~/.cargo/bin is in your PATH.\n");
    println!("After installation, you can run 'feedtui' from anywhere!");

    Ok(())
}

async fn youtube_auth(client_id: String, client_secret: String) -> Result<()> {
    println!("=== YouTube Authentication ===\n");
    println!("This will authenticate feedtui with your YouTube account");
    println!("to access personalized feeds (subscriptions, liked videos, etc.).\n");

    let oauth = YouTubeOAuth::new(client_id, client_secret);

    // Check if already authenticated
    if oauth.has_valid_tokens() {
        println!("You are already authenticated with YouTube.");
        println!("Run 'feedtui youtube-logout' to clear tokens and re-authenticate.\n");
        return Ok(());
    }

    // Start device flow
    println!("Starting device authorization flow...\n");
    let device_auth = oauth.start_device_flow().await?;

    println!("Please visit: {}", device_auth.verification_url);
    println!("And enter code: {}\n", device_auth.user_code);
    println!("Waiting for authorization...");

    // Poll for token
    match oauth
        .poll_for_token(&device_auth.device_code, device_auth.interval)
        .await
    {
        Ok(_tokens) => {
            println!("\nAuthentication successful!");
            println!("YouTube tokens have been saved.");
            println!("\nYou can now use personalized feed types in your config:");
            println!("  feed_type = \"subscriptions\"   # Your subscription feed");
            println!("  feed_type = \"liked_videos\"    # Your liked videos");
            println!("  feed_type = \"watch_later\"     # Your Watch Later playlist");
        }
        Err(e) => {
            eprintln!("\nAuthentication failed: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

fn youtube_logout() -> Result<()> {
    println!("=== YouTube Logout ===\n");

    // Use a dummy OAuth instance just for clearing tokens
    let oauth = YouTubeOAuth::new(String::new(), String::new());

    match oauth.clear_tokens() {
        Ok(_) => {
            println!("YouTube authentication tokens have been cleared.");
            println!("Run 'feedtui youtube-auth' to re-authenticate.");
        }
        Err(e) => {
            eprintln!("Failed to clear tokens: {}", e);
            return Err(e);
        }
    }

    Ok(())
}
