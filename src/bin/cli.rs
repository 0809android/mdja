use mdja::Document;
use std::env;
use std::fs;
use std::io::{self, Read};
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args
        .get(1)
        .is_some_and(|arg| arg == "-h" || arg == "--help")
    {
        eprintln!("使い方: mdja <input.md> [output.html]");
        eprintln!("\n例:");
        eprintln!("  mdja input.md              # HTMLを標準出力に表示");
        eprintln!("  mdja input.md output.html  # HTMLをファイルに保存");
        eprintln!("  cat input.md | mdja        # 標準入力から読み込み");
        eprintln!("  cat input.md | mdja -      # 標準入力から読み込み");
        return;
    }

    // Markdownを読み込み
    let markdown = if args.len() < 2 || args[1] == "-" {
        // 標準入力から読み込み
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .expect("標準入力の読み込みに失敗しました");
        buffer
    } else {
        // ファイルから読み込み
        fs::read_to_string(&args[1]).unwrap_or_else(|e| {
            eprintln!("エラー: ファイルの読み込みに失敗: {}", e);
            process::exit(1);
        })
    };

    // パース
    let doc = Document::parse(&markdown);

    // 出力
    if args.len() >= 3 {
        // ファイルに保存
        fs::write(&args[2], &doc.html).unwrap_or_else(|e| {
            eprintln!("エラー: ファイルの書き込みに失敗: {}", e);
            process::exit(1);
        });
        println!("✓ {}に保存しました", args[2]);

        if !doc.metadata.is_empty() {
            println!("\nメタデータ:");
            for (key, value) in &doc.metadata {
                println!("  {}: {}", key, value);
            }
        }

        println!("\n読了時間: {}分", doc.reading_time);
        println!("見出し数: {}", doc.headings.len());
    } else {
        // 標準出力
        println!("{}", doc.html);
    }
}
