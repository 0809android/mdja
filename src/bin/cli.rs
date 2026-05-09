use mdja::{AnchorStyle, Document, ParseOptions};
use std::env;
use std::fs;
use std::io::{self, Read};
use std::process;
use std::thread;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputMode {
    Html,
    Json,
    Toc,
    TocHtml,
    Metadata,
    ReadingTime,
    Headings,
}

struct Cli {
    input: Option<String>,
    output: Option<String>,
    output_mode: OutputMode,
    parse_options: ParseOptions,
    watch: bool,
}

fn main() {
    let cli = parse_args(env::args().skip(1).collect()).unwrap_or_else(|e| {
        eprintln!("エラー: {e}");
        print_help();
        process::exit(1);
    });

    if cli.watch {
        watch_file(&cli);
        return;
    }

    let markdown = read_markdown(cli.input.as_deref());
    let output = render_output(&markdown, &cli);
    write_output(cli.output.as_deref(), &output);
}

fn parse_args(args: Vec<String>) -> Result<Cli, String> {
    let mut cli = Cli {
        input: None,
        output: None,
        output_mode: OutputMode::Html,
        parse_options: ParseOptions::default(),
        watch: false,
    };

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help();
                process::exit(0);
            }
            "--json" => cli.output_mode = OutputMode::Json,
            "--toc" => cli.output_mode = OutputMode::Toc,
            "--toc-html" => cli.output_mode = OutputMode::TocHtml,
            "--metadata" => cli.output_mode = OutputMode::Metadata,
            "--reading-time" => cli.output_mode = OutputMode::ReadingTime,
            "--headings" => cli.output_mode = OutputMode::Headings,
            "--watch" => cli.watch = true,
            "--toc-min-level" => {
                i += 1;
                cli.parse_options.toc_min_level = parse_usize_arg(&args, i, "--toc-min-level")?;
            }
            "--toc-max-level" => {
                i += 1;
                cli.parse_options.toc_max_level = parse_usize_arg(&args, i, "--toc-max-level")?;
            }
            "--reading-speed-ja" => {
                i += 1;
                cli.parse_options.reading_speed_japanese =
                    parse_usize_arg(&args, i, "--reading-speed-ja")?;
            }
            "--reading-speed-en" => {
                i += 1;
                cli.parse_options.reading_speed_english =
                    parse_usize_arg(&args, i, "--reading-speed-en")?;
            }
            "--anchor-style" => {
                i += 1;
                cli.parse_options.anchor_style = parse_anchor_style_arg(&args, i)?;
            }
            arg if arg.starts_with('-') && arg != "-" => {
                return Err(format!("不明なオプション: {arg}"));
            }
            arg => {
                if cli.input.is_none() {
                    cli.input = Some(arg.to_string());
                } else if cli.output.is_none() {
                    cli.output = Some(arg.to_string());
                } else {
                    return Err(format!("余分な引数です: {arg}"));
                }
            }
        }
        i += 1;
    }

    Ok(cli)
}

fn parse_usize_arg(args: &[String], index: usize, name: &str) -> Result<usize, String> {
    args.get(index)
        .ok_or_else(|| format!("{name} には数値が必要です"))?
        .parse()
        .map_err(|_| format!("{name} には正の整数を指定してください"))
}

fn parse_anchor_style_arg(args: &[String], index: usize) -> Result<AnchorStyle, String> {
    match args.get(index).map(String::as_str) {
        Some("romaji") => Ok(AnchorStyle::Romaji),
        Some("ascii") => Ok(AnchorStyle::Ascii),
        Some(value) => Err(format!(
            "--anchor-style は romaji または ascii です: {value}"
        )),
        None => Err("--anchor-style には romaji または ascii が必要です".to_string()),
    }
}

fn read_markdown(input: Option<&str>) -> String {
    let Some(input) = input.filter(|input| *input != "-") else {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .expect("標準入力の読み込みに失敗しました");
        return buffer;
    };

    fs::read_to_string(input).unwrap_or_else(|e| {
        eprintln!("エラー: ファイルの読み込みに失敗: {e}");
        process::exit(1);
    })
}

fn render_output(markdown: &str, cli: &Cli) -> String {
    let doc = Document::parse_with_options(markdown, &cli.parse_options);
    match cli.output_mode {
        OutputMode::Html => doc.html,
        OutputMode::Json => serde_json::to_string_pretty(&doc).unwrap_or_else(|e| {
            eprintln!("エラー: JSON変換に失敗: {e}");
            process::exit(1);
        }),
        OutputMode::Toc => doc.toc,
        OutputMode::TocHtml => doc.toc_html,
        OutputMode::Metadata => {
            serde_json::to_string_pretty(&doc.metadata_raw).unwrap_or_else(|e| {
                eprintln!("エラー: メタデータのJSON変換に失敗: {e}");
                process::exit(1);
            })
        }
        OutputMode::ReadingTime => format!("{}\n", doc.reading_time),
        OutputMode::Headings => serde_json::to_string_pretty(&doc.headings).unwrap_or_else(|e| {
            eprintln!("エラー: 見出しのJSON変換に失敗: {e}");
            process::exit(1);
        }),
    }
}

fn write_output(output_path: Option<&str>, output: &str) {
    if let Some(path) = output_path {
        fs::write(path, output).unwrap_or_else(|e| {
            eprintln!("エラー: ファイルの書き込みに失敗: {e}");
            process::exit(1);
        });
    } else {
        print!("{output}");
    }
}

fn watch_file(cli: &Cli) {
    let Some(input) = cli.input.as_deref().filter(|input| *input != "-") else {
        eprintln!("エラー: --watch には入力ファイルが必要です");
        process::exit(1);
    };

    let mut last_modified: Option<SystemTime> = None;
    loop {
        let modified = fs::metadata(input).and_then(|metadata| metadata.modified());
        match modified {
            Ok(modified) if last_modified != Some(modified) => {
                last_modified = Some(modified);
                let markdown = read_markdown(Some(input));
                let output = render_output(&markdown, cli);
                write_output(cli.output.as_deref(), &output);
                eprintln!("updated: {input}");
            }
            Ok(_) => {}
            Err(e) => eprintln!("エラー: ファイル監視に失敗: {e}"),
        }
        thread::sleep(Duration::from_secs(1));
    }
}

fn print_help() {
    eprintln!("使い方: mdja [options] [input.md|-] [output]");
    eprintln!();
    eprintln!("出力:");
    eprintln!("  --json              Document全体をJSONで出力");
    eprintln!("  --toc               Markdown形式のTOCを出力");
    eprintln!("  --toc-html          HTML形式のTOCを出力");
    eprintln!("  --metadata          frontmatterをJSONで出力");
    eprintln!("  --reading-time      読了時間だけを出力");
    eprintln!("  --headings          見出し一覧をJSONで出力");
    eprintln!();
    eprintln!("オプション:");
    eprintln!("  --toc-min-level N       TOCの最小見出しレベル");
    eprintln!("  --toc-max-level N       TOCの最大見出しレベル");
    eprintln!("  --reading-speed-ja N    日本語の読了速度（文字/分）");
    eprintln!("  --reading-speed-en N    英語の読了速度（単語/分）");
    eprintln!("  --anchor-style STYLE    romaji または ascii");
    eprintln!("  --watch                 入力ファイルを監視して再出力");
    eprintln!();
    eprintln!("例:");
    eprintln!("  mdja input.md output.html");
    eprintln!("  mdja --json input.md");
    eprintln!("  mdja --toc --toc-max-level 3 input.md");
    eprintln!("  cat input.md | mdja --metadata");
}
