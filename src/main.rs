mod model;

use crate::model::*;

use std::{
    collections::HashMap,
    env,
    io::{self, Write},
    path::Path,
};

use async_recursion::async_recursion;
use chrono::Datelike;
use clap::Parser;
use colored::Colorize;
use eyre::{bail, Context, Result};
use regex::{Captures, Regex};
use reqwest::header::HeaderMap;
use tokio::{fs, try_join};

const PATH_EXAMPLE: &str = "./example.txt";
const PATH_INPUT: &str = "./input.txt";
const PATH_PROBLEM: &str = "./problem.md";

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config = load_config(&args).await;

    let session = env::var("AOC_SESSION").context("AOC_SESSION not defined in environment")?;

    let mut headers = HeaderMap::new();
    headers.insert("cookie", format!("session={session}").parse()?);
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    match args.command {
        Command::New {
            output,
            year,
            day,
            template,
        } => {
            let year = year.unwrap_or_else(get_default_year);
            let output = output.unwrap_or_else(|| format!("./day_{day}").into());
            create_project(year, day, &template, &output).await?;
            env::set_current_dir(&output)?;
            let (problem, _) = try_join!(
                download_problem(&client, year, day, PATH_PROBLEM),
                download_input(&client, year, day, PATH_INPUT),
            )?;
            println!("\n{}", "Problem:".cyan());
            println!("{problem}\n\n");
            let examples = download_potential_examples(&client, year, day).await?;
            choose_and_save_correct_example(examples, PATH_EXAMPLE).await?;
        }
        Command::Download { example, year, day } => {
            let year = get_year(year, &config)?;
            let day = get_day(day, &config)?;
            if example {
                let examples = download_potential_examples(&client, year, day).await?;
                choose_and_save_correct_example(examples, PATH_EXAMPLE).await?;
            } else {
                let (problem, _) = try_join!(
                    download_problem(&client, year, day, PATH_PROBLEM),
                    download_input(&client, year, day, PATH_INPUT),
                )?;
                println!("\n{}", "Problem:".cyan());
                println!("{problem}\n\n");
            }
        }
        Command::Submit {
            result,
            year,
            day,
            level,
        } => {
            let year = get_year(year, &config)?;
            let day = get_day(day, &config)?;
            let level = get_level(level).await?;
            let response = submit_input(&client, year, day, level, &result).await?;
            if !response.contains("not the right answer") {
                download_problem(&client, year, day, PATH_PROBLEM).await?;
            }
            println!("\nResponse:\n{response}");
        }
    }

    Ok(())
}

fn get_year(arg: Option<u32>, config: &Option<Config>) -> Result<u32> {
    if let Some(arg) = arg {
        return Ok(arg);
    }
    if let Some(config) = config {
        return Ok(config.year);
    }
    bail!("Missing argument 'year'");
}

fn get_default_year() -> u32 {
    let now = chrono::Utc::now();
    let year = now.year() as u32;
    if now.month() == 12 {
        year
    } else {
        year - 1
    }
}

fn get_day(arg: Option<u32>, config: &Option<Config>) -> Result<u32> {
    if let Some(arg) = arg {
        return Ok(arg);
    }
    if let Some(config) = config {
        return Ok(config.day);
    }
    bail!("Missing argument 'day'");
}

async fn get_level(arg: Option<u32>) -> Result<u32> {
    if let Some(arg) = arg {
        return Ok(arg);
    }

    println!("Try to guess level based on '{PATH_PROBLEM}'.");
    let problem_text = fs::read_to_string(PATH_PROBLEM)
        .await
        .context(format!("Error while opening '{PATH_PROBLEM}'"))?;

    match problem_text.contains("--- Part Two ---") {
        true => Ok(2),
        false => Ok(1),
    }
}

async fn load_config(args: &Args) -> Option<Config> {
    let path = &args.config;
    let file_content = fs::read_to_string(&path).await.ok()?;
    let config: Config = match toml::from_str(&file_content) {
        Ok(config) => config,
        Err(err) => {
            eprintln!(
                "Failed to parse config {path}: {err}",
                path = &path.to_string_lossy()
            );
            return None;
        }
    };
    Some(config)
}

async fn create_project(year: u32, day: u32, template: &Path, output: &Path) -> Result<()> {
    let output_str = output.to_string_lossy();
    let template_str = template.to_string_lossy();
    println!("Copy '{template_str}' to '{output_str}'");
    copy_dir_all(template, output)
        .await
        .context(format!("Failed to copy '{template_str}' to '{output_str}'"))?;

    // Replace project name for rust projects
    let cargo_toml = &output.join("Cargo.toml");
    if let Ok(content) = fs::read_to_string(cargo_toml).await {
        let cargo_toml_str = cargo_toml.to_string_lossy();
        println!("Change name in '{cargo_toml_str}'");
        fs::write(
            cargo_toml,
            content.replace(r#"name = "template""#, &format!(r#"name = "day_{day}""#)),
        )
        .await
        .context(format!("Failed to update '{cargo_toml_str}'"))?;
    }

    // Add aoc config
    let aoc_toml = &output.join("aoc.toml");
    let aoc_toml_str = aoc_toml.to_string_lossy();
    let config = Config { year, day };
    println!("Generate '{aoc_toml_str}'");
    fs::write(aoc_toml, toml::to_string_pretty(&config)?)
        .await
        .context(format!("Failed to write '{}'", aoc_toml.to_string_lossy()))?;

    Ok(())
}

async fn download_problem(
    client: &reqwest::Client,
    year: u32,
    day: u32,
    output_file: &str,
) -> Result<String> {
    let url = format!("https://adventofcode.com/{year}/day/{day}");
    let html = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let article = format_html_output(&html)?;
    save(output_file, &article).await?;
    Ok(article)
}

async fn download_input(
    client: &reqwest::Client,
    year: u32,
    day: u32,
    output_file: &str,
) -> Result<()> {
    let url = format!("https://adventofcode.com/{year}/day/{day}/input");
    let input_text = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    save(output_file, &input_text).await?;
    Ok(())
}

async fn submit_input(
    client: &reqwest::Client,
    year: u32,
    day: u32,
    level: u32,
    result: &str,
) -> Result<String> {
    let url = format!("https://adventofcode.com/{year}/day/{day}/answer");
    let mut form = HashMap::new();
    form.insert("level", format!("{level}"));
    form.insert("answer", result.into());
    let response = client
        .post(url)
        .form(&form)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let response = format_html_output(&response)?;
    Ok(response)
}

async fn download_potential_examples(
    client: &reqwest::Client,
    year: u32,
    day: u32,
) -> Result<Vec<String>> {
    let url = format!("https://adventofcode.com/{year}/day/{day}");
    let html = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let dom = tl::parse(&html, Default::default())?;
    let parser = dom.parser();
    let examples: Vec<String> = dom
        .query_selector("pre")
        .unwrap()
        .map(|element| element.get(parser).unwrap())
        .map(|node| {
            node.inner_text(parser)
                .to_string()
                .replace("&lt;", "<")
                .replace("&gt;", ">")
        })
        .collect();
    Ok(examples)
}

async fn choose_and_save_correct_example(examples: Vec<String>, output_file: &str) -> Result<()> {
    if examples.is_empty() {
        println!("{}", "No examples found.".cyan());
        return Ok(());
    }

    println!(
        "{}",
        format!(
            "Downloaded {} potential examples, please choose one:",
            examples.len()
        )
        .cyan()
    );
    for (i, example) in examples.iter().enumerate().rev() {
        println!();
        println!("{}", format!("Example {}:", i).cyan());
        let lines: Vec<_> = example.lines().collect();
        let short_example = limit_size(&lines, 10).join("\n");
        println!("{}", short_example);
    }
    println!();
    print!("{}", "> Choose example: ".cyan());
    io::stdout().flush()?;
    match read_user_input()?.parse::<usize>() {
        Ok(n) if n < examples.len() => {
            save(output_file, &examples[n]).await?;
            Ok(())
        }
        _ => {
            println!();
            println!("{}", "Nothing selected".cyan());
            Ok(())
        }
    }
}

fn limit_size<T>(list: &[T], limit: usize) -> &[T] {
    list.get(..limit).unwrap_or(list)
}

fn format_html_output(html: &str) -> Result<String> {
    let dom = tl::parse(html, Default::default())?;
    let parser = dom.parser();
    let articles: Vec<_> = dom
        .query_selector("article")
        .unwrap()
        .map(|node| node.get(parser).unwrap().inner_html(parser))
        .collect();
    let articles = articles.join("\n");
    let html = format!("<div>{articles}</div>");
    let article = html_to_text(&html);
    Ok(article)
}

fn html_to_text(html: &str) -> String {
    let text = html2text::from_read(html.as_bytes(), 80);

    // Wrap multiline code examples with triple ```
    let code_regex = Regex::new(r"`([^`]+)`").unwrap();
    let text = code_regex.replace_all(&text, |caps: &Captures| {
        let content = &caps[1];
        if content.contains('\n') {
            format!("```\n{content}\n```")
        } else {
            format!("`{content}`")
        }
    });

    text.to_string()
}

fn read_user_input() -> io::Result<String> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

async fn save(file: &str, content: &str) -> Result<()> {
    println!("{}", format!("Saving {file}").cyan());
    fs::write(file, content)
        .await
        .with_context(|| format!("Couldn't write to {file}"))?;
    Ok(())
}

#[async_recursion]
async fn copy_dir_all<S, D>(src: S, dst: D) -> io::Result<()>
where
    S: AsRef<Path> + Send + Sync,
    D: AsRef<Path> + Send + Sync,
{
    fs::create_dir_all(&dst).await?;
    let mut read_dir = fs::read_dir(src).await?;
    while let Ok(Some(entry)) = read_dir.next_entry().await {
        let file_type = entry.file_type().await?;
        if file_type.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name())).await?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name())).await?;
        }
    }
    Ok(())
}
