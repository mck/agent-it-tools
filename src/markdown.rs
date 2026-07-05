use crate::util::read_input;
use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum MarkdownCmd {
    /// Render Markdown (GitHub-flavored: tables, strikethrough, task lists, autolinks) to HTML
    ToHtml {
        /// Markdown input (reads stdin if omitted)
        input: Option<String>,
    },
    /// Convert HTML to Markdown
    FromHtml {
        /// HTML input (reads stdin if omitted)
        input: Option<String>,
    },
}

pub fn run(cmd: MarkdownCmd) -> Result<()> {
    match cmd {
        MarkdownCmd::ToHtml { input } => {
            let input = read_input(input)?;
            let mut options = comrak::Options::default();
            options.extension.table = true;
            options.extension.strikethrough = true;
            options.extension.autolink = true;
            options.extension.tasklist = true;
            print!("{}", comrak::markdown_to_html(&input, &options));
        }
        MarkdownCmd::FromHtml { input } => {
            let input = read_input(input)?;
            let md =
                htmd::convert(&input).map_err(|e| anyhow::anyhow!("cannot convert HTML: {e}"))?;
            println!("{}", md.trim_end());
        }
    }
    Ok(())
}
