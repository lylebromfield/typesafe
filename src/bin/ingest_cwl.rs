use std::fs::File;
use std::io::Write;
use serde::Serialize;
use std::collections::HashSet;

#[derive(Serialize)]
struct LatexItem {
    trigger: String,
    completion: String,
}

#[derive(Serialize)]
struct LatexData {
    commands: Vec<LatexItem>,
    environments: Vec<LatexItem>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting CWL ingestion...");

    let files = vec![
        "tex.cwl",
        "latex-209.cwl",
        "latex-document.cwl",
        "latex-mathsymbols.cwl",
        "amsmath.cwl",
        "graphicx.cwl",
        "hyperref.cwl",
        "biblatex.cwl",
        // Extended Packages
        "algorithm2e.cwl",
        "algorithms.cwl",
        "amssymb.cwl",
        "amsthm.cwl",
        "array.cwl",
        "babel.cwl",
        "beamer.cwl",
        "booktabs.cwl",
        "caption.cwl",
        "cleveref.cwl",
        "color.cwl",
        "csquotes.cwl",
        "enumitem.cwl",
        "etoolbox.cwl",
        "fancyhdr.cwl",
        "float.cwl",
        "fontspec.cwl",
        "geometry.cwl",
        "glossaries.cwl",
        "listings.cwl",
        "longtable.cwl",
        "mathtools.cwl",
        "microtype.cwl",
        "minted.cwl",
        "multicol.cwl",
        "multirow.cwl",
        "natbib.cwl",
        "pgfplots.cwl",
        "siunitx.cwl",
        "subcaption.cwl",
        "tabularx.cwl",
        "tcolorbox.cwl",
        "tikz.cwl",
        "titlesec.cwl",
        "tocbibind.cwl",
        "ulem.cwl",
        "url.cwl",
        "xcolor.cwl",
        "xparse.cwl",
    ];

    let base_url = "https://raw.githubusercontent.com/texstudio-org/texstudio/master/completion";

    let mut commands = Vec::new();
    let mut environments = Vec::new();
    let mut seen_triggers = HashSet::new();

    for filename in files {
        let url = format!("{}/{}", base_url, filename);
        println!("Fetching {}...", url);

        let resp = reqwest::blocking::get(&url)?.text()?;

        for line in resp.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse CWL line
            // \command{arg1}{arg2}#type

            // 1. Separate classifier
            let (content, _classifier) = if let Some(idx) = line.rfind('#') {
                (&line[..idx], &line[idx..])
            } else {
                (line, "")
            };

            // 2. Identify if it's an environment definition
            // Usually defined as \begin{env} or just "env" in some contexts, but CWL usually lists \begin{env} explicitly?
            // Actually, TeXStudio CWL often lists `\begin{environment}` for completion.
            // But they also list just `\env` for the command form.

            let is_env = content.starts_with("\\begin{");

            if is_env {
                // Handle environment definition
                if let Some(close_idx) = content.find('}') {
                    let env_name = &content[7..close_idx]; // skip \begin{
                    let trigger = env_name.to_string();

                    if seen_triggers.contains(&trigger) {
                        continue;
                    }
                    seen_triggers.insert(trigger.clone());

                    // Check for arguments after \begin{env}
                    let rest = &content[close_idx+1..];
                    let mut args_snippet = String::new();

                    let mut chars = rest.chars().peekable();
                    while let Some(c) = chars.next() {
                        if c == '{' || c == '[' {
                            let terminator = if c == '{' { '}' } else { ']' };
                            while let Some(ac) = chars.next() {
                                if ac == terminator { break; }
                            }
                            args_snippet.push(c);
                            args_snippet.push(terminator);
                        }
                    }

                    let completion = if args_snippet.is_empty() {
                        format!("\\begin{{{}}}\n\t\n\\end{{{}}}", env_name, env_name)
                    } else {
                        format!("\\begin{{{}}}{}\n\t\n\\end{{{}}}", env_name, args_snippet, env_name)
                    };

                    environments.push(LatexItem {
                        trigger,
                        completion,
                    });
                }
            } else {
                // 3. Parse arguments into snippets
                let mut trigger = String::new();
                let mut completion = String::new();

                // Extract the command name for trigger
                // e.g. \frac{num}{den} -> trigger: \frac

                let mut chars = content.chars().peekable();
                let mut in_command = true;

                while let Some(c) = chars.next() {
                    if in_command {
                        if c == '{' || c == '[' {
                            in_command = false;
                            // Start of first arg
                            trigger = completion.clone(); // The trigger is what we parsed so far
                            completion.push(c);

                            // Slurp until closing
                            let terminator = if c == '{' { '}' } else { ']' };
                            while let Some(ac) = chars.next() {
                                if ac == terminator {
                                    break;
                                }
                            }

                            // Format snippet
                            completion.push(terminator);
                        } else {
                            completion.push(c);
                        }
                    } else {
                        // Processing subsequent args
                        if c == '{' || c == '[' {
                            completion.push(c);
                            let terminator = if c == '{' { '}' } else { ']' };
                            while let Some(ac) = chars.next() {
                                if ac == terminator {
                                    break;
                                }
                            }
                             completion.push(terminator);
                        } else {
                            completion.push(c);
                        }
                    }
                }

                if trigger.is_empty() {
                    trigger = completion.clone();
                }

                // Filter duplicates
                if seen_triggers.contains(&trigger) {
                    continue;
                }
                seen_triggers.insert(trigger.clone());

                let item = LatexItem {
                    trigger: trigger.clone(),
                    completion: completion.clone(),
                };
                commands.push(item);
            }
        }
    }

    let data = LatexData {
        commands,
        environments,
    };

    let json = serde_json::to_string_pretty(&data)?;
    let mut file = File::create("latex_data.json")?;
    file.write_all(json.as_bytes())?;

    println!("Done! Generated latex_data.json with {} commands and {} environments.", data.commands.len(), data.environments.len());

    Ok(())
}
