


use std::string::FromUtf8Error;
extern crate ctrlc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use std::process;


use std::fs::File;
use std::fs;
use std::env;
use std::process::{
    Command,
    Output
};
use std::io::{
    Error,
    Write
};

use std::env::temp_dir;
use std::path::PathBuf;
// 定数の宣言
const exit_string : &str = "exit";

fn main() {
    let mut default_command : String = "php".to_string();
    // 実行時のコマンドライン引数を取得
    let arguments: Vec<String> = env::args().collect();
    if (arguments.len() < 2) {
        panic!("Select any execute file.");
    }
    let command= arguments.get(1);
    // 入力されたコマンドが php | ruby | python?
    let mut initialize_input : String = "".to_string();
    match command {
        default_command => {
            initialize_input = "<?php \r\n".to_string();
        },
        _ => {
            initialize_input.clear();
        }
    }


    // 起動中の自身のプロセスID
    let my_pid: u32 =  process::id();

    // 改行された回数を保持
    let mut previous_newline_count : i32 = 0;
    let mut current_newline_count : i32 = 0;

    // 検証用ソース・ファイルと実行用ソース・ファイルの2つのFileオブジェクトを取得する
    let mut validate_dir : PathBuf= env::temp_dir();
    validate_dir.push("validate_log.dat");
    let validate_file_path = format!("{}", validate_dir.display());
    let validate_file : Result<File, Error> = File::create(&validate_file_path);

    // 実行用
    let mut execute_dir: PathBuf = env::temp_dir();
    execute_dir.push("execute_log.dat");
    let execute_file_path = format!("{}", execute_dir.display());
    let execute_file : Result<File, Error> = File::create(&execute_file_path);

    // 検証用ソース・ファイルの作成失敗時
    if (validate_file.is_ok() != true) {
        panic!("{}", validate_file.unwrap_err());
        process::exit(my_pid as i32);
    }
    let mut validate_file = validate_file.unwrap();
    // 検証用ファイルの初期化
    validate_file.write_all(initialize_input.as_bytes());

    // 実行用ソース・ファイルの作成失敗時
    if (execute_file.is_ok() != true) {
        panic!("{}", execute_file.unwrap_err().to_string());
        process::exit(my_pid as i32);
    }
    let mut execute_file = execute_file.unwrap();


    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    println!("Waiting for Ctrl-C...");
    // while running.load(Ordering::SeqCst) {}
    println!("Got it! Exiting...");

    // コマンドラインからの入力を取
    let mut input : String = String::new();

    // 書き込み時の戻り値を保持
    let mut written_bytes : Result<(), Error>;

    while (true) {
        let mut input_data = std::io::stdin().read_line(&mut input);
        if input_data.is_ok() != true {
            panic!("error => {}", input_data.unwrap_err());
        }
        remove_newline(&mut input);
        // コマンドラインを終了するための処理
        if (exit_string.to_string() == input) {
            break;
        }
        let target_index : i32= input.len() as i32 - 1;
        if str_position(&input, '\\' as u8) == target_index {
            continue;
        }


        if (input.len() > 0) {
            input.push_str("\n print(\"\n\"); \n");
        } else {
            continue;
        }
        validate_file.write_all(input.as_bytes());



        // 以下よりphpコマンドの実行
        let mut output_result : Result<Output, Error>;
        let mut output : Output;
        let mut output_vector : Vec<u8>;
        if cfg!(windows) {
            output_result = Command::new(&default_command).args(&[&validate_file_path]).output();
            if (output_result.is_ok() != true) {
                panic!("{}", output_result.unwrap_err().to_string());
            }
        } else {
            panic!("Your machine is unsupported.");
        }

        output = output_result.unwrap();
        let mut exit_code : Option<i32> = output.status.code();
        let mut for_output : Vec<u8> = Vec::new();
        // コマンドの実行結果が 「0」かどうかを検証
        if (exit_code.is_some() == true && exit_code.unwrap() == 0) {

            // 検証用ファイルでプログラムが正常終了した場合
            let temp_file : String = fs::read_to_string(&validate_file_path).unwrap();
            execute_file.set_len(0);
            written_bytes = execute_file.write_all(temp_file.as_bytes());
            if (written_bytes.is_ok() != true) {
                panic!("Err: {}", written_bytes.unwrap_err().to_string());
            }

            // 実行用ファイルで再度コマンド実行
            output_result = Command::new(&default_command).args(&[&execute_file_path]).output();
            if (output_result.is_ok() != true) {
                panic!("{}", output_result.unwrap_err().to_string());
            }
            output = output_result.unwrap();
            let mut exit_code : Option<i32> = output.status.code();
            let mut for_output : Vec<u8> = Vec::new();

            for value in output.stdout {
                // 前回まで出力した分は破棄する
                if (previous_newline_count <= current_newline_count) {
                    for_output.push(value);
                }
                if (value == 10) {
                    // println!("*value => {}", *value);
                    // println!("&value => {}", value);
                    current_newline_count = current_newline_count + 1;
                }
            }

            if String::from_utf8(for_output.clone()).is_ok() == true {
                println!("output:> {} \n", String::from_utf8(for_output).unwrap());
            } else {
                println!("Err1: Failed to be executed the command which you input on background!");
                println!("Err1: {}", String::from_utf8(for_output).unwrap_err());
            }

            previous_newline_count = current_newline_count;
            current_newline_count = 0;
        } else {
            // 入力したプログラムの実行が失敗した場合
            let mut backup_contents : Result<String, Error>;
            backup_contents = fs::read_to_string(&execute_file_path);
            if (backup_contents . is_ok() == true) {
                let backup_contents = backup_contents.unwrap();
                validate_file.set_len(0);
                validate_file.write_all(backup_contents.as_bytes());
            }

            println!("Err2: Failed to be executed the command which you input on background!");
            println!("Err2: {}", String::from_utf8(output.stderr).unwrap());
        }
        input.clear();
    }
    // 終了コマンド実行後----
    println!("See you again!");
    // u32型から => i32型へキャスト
    process::exit(my_pid as i32);
}



// テキストに任意の文字列が含まれているかどうか
// 含まれない場合 -1の負の数を返却する
fn str_position (target : &String, needle : u8) -> i32
{
    check_type(target.as_bytes().iter().enumerate());
    for (index, value) in target.as_bytes().iter().enumerate() {
        // println!("index => {}", index);
        // println!("value => {}", value);
        // println!("needle => {}", needle);
        if (needle == *value) {
            return (index as i32);
        }
    }
    return -1;
}


// 末尾の改行文字を削除
fn remove_newline(newline_string : &mut String)
{
    if (newline_string.len() == 0) {
        return ();
    }
    let new_vec : Vec<u8>;
    unsafe {
        {
            let _bytes = newline_string.as_mut_vec();
            // println!("{}", _bytes[_bytes.len() -1]);
            if (_bytes[_bytes.len() -1] == 10) {
                _bytes.pop();
            }
            if (_bytes[_bytes.len() - 1] == 13) {
                _bytes.pop();
            }
            check_type(&_bytes);
            new_vec = _bytes.to_vec();
            check_type(&_bytes);
            check_type(&new_vec);
        }
        let response : Result<String, FromUtf8Error> = String::from_utf8(new_vec);
        if (response.is_ok() == true) {
            *newline_string = response.unwrap();
        } else {
            *newline_string = response.unwrap_err().to_string();
        }
    }
}

fn check_type<T>(_: T) -> String {
    println!("{}", std::any::type_name::<T>());
    return std::any::type_name::<T>().to_string();
}