


use std::string::FromUtf8Error;
extern crate ctrlc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use std::process;
use std::fs;
use std::env;
use std::process::{
    Command,
    Output,
    Stdio,
};
use std::io::{
    Error,
    Write
};

use std::env::temp_dir;
use std::path::PathBuf;


// 定数の宣言
const EXIT_STRING : &str = "exit";
const CLEAR_STRING: &str = "clear";
const DEL_STRING: &str = "del";


use std::ffi::{
    CString,
    CStr
};
use std::os::raw::{
    c_char,
    c_int,
    c_void
};

use std::io::prelude::*;


// echoモジュールを使用
mod echo;
// echoモジュール内のecho関数を単独で利用する
use echo::*;

use std::io::BufReader;

use printf_rs::*;

/// ファイルからbytesを読み込んでいく。
fn get_file_resource (path: &String) -> Vec<u8> {
    let mut read_bytes : Vec<u8> = Vec::new();
    let f : Result<std::fs::File, Error> = fs::File::open(path);
    if f.is_ok() != true {
        /// panicせずにエラーのみを表示
        println!("Err {}", f.unwrap_err());
        return read_bytes;
    }
    let f = f.unwrap();
    let byte_list = f.bytes();
    for value in byte_list {
        read_bytes.push(value.unwrap());
    }
    return read_bytes;
}

pub fn printf_c_string(output : Vec<u8>) -> isize {

    unsafe {
        extern "C" {
            fn printf(format: *const c_char, args: *mut c_void) -> String;
        }

        let c_percent = (CString::new("%s".to_string()).unwrap().as_ptr()) as *const c_char;

        let c_string = (CString::new(output).unwrap().as_ptr()) as *mut c_void;

        printf(c_percent, c_string);
    }
    return -1;
}


fn main() {

    // printf!("%s \n", cstr!("Hello World !")); // print string
    // printf!("%i \n", 1234); // print integer
    printf_c_string(get_file_resource(&"./shift_jis.dat".to_string()));


    let default_command : String = "php".to_string();
    // 実行時のコマンドライン引数を取得
    let arguments: Vec<String> = env::args().collect();
    let command: String;
    if arguments.len() >= 2
    {
        command = arguments.get(1).unwrap().to_string();
    } else {
        command = "php".to_string();
        // panic!("Select any execute file.");
    }
    // 入力されたコマンドが php | ruby | python?
    let mut initialize_input : String;

    initialize_input = "".to_string();
    if command == default_command {
        initialize_input = "<?php \r\n".to_string();
    }



    // 起動中の自身のプロセスID
    let my_pid: u32 =  process::id();

    // 改行された回数を保持
    let mut previous_newline_count : i32 = 0;
    let mut current_newline_count : i32 = 0;

    // 検証用ソース・ファイルと実行用ソース・ファイルの2つのFileオブジェクトを取得する
    let mut validate_dir : PathBuf = env::temp_dir();
    validate_dir.push("validate_log.dat");
    let validate_file_path = format!("{}", validate_dir.display());

    // ファイル作成関数を利用する
    let mut validate_file : std::fs::File = create_new_file(&validate_file_path);


    // 実行用
    let mut execute_dir: PathBuf = env::temp_dir();
    execute_dir.push("execute_log.dat");
    let execute_file_path = format!("{}", execute_dir.display());
    // ファイル作成関数を利用する
    let mut execute_file : std::fs::File = create_new_file( &execute_file_path);
    // 検証用ファイルの初期化
    validate_file.write_all(initialize_input.as_bytes());



    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    println!("Input any source code with {}.", command);
    // while running.load(Ordering::SeqCst) {}

    // コマンドラインからの入力を取
    let mut input : String = String::new();

    // 書き込み時の戻り値を保持
    let mut written_bytes : Result<(), Error>;
    let mut line_number: usize = 0;
    while (true) {
        println!("{}:", line_number = line_number + 1);
        line_number = line_number + 1;
        let input_data : Result<usize, Error> = std::io::stdin().read_line(&mut input);

        // コマンドラインから入力されたbyte数を取得する
        if input_data.is_ok() != true
        {
            panic!("error => {}", input_data.unwrap_err());
        }
        // 入力内容から、改行文字を削除
        remove_newline(&mut input);



        // 入力内容によってループ内の処理を変更する
        // コマンドラインを終了するための処理
        if (EXIT_STRING.to_string() == input) {
            // exit
            break;
        } else if (CLEAR_STRING.to_string() == input ) {
            // 正常実行が完了していた直近のソースコードまで巻き上げる
            let backup_string : String = fs::read_to_string(&execute_file_path).unwrap();
            // 検証用ファイルを一旦削除
            remove_target_file(&validate_file_path);
            // 再度検証用ファイルを作成
            validate_file = create_new_file(&validate_file_path);
            validate_file.write_all(backup_string.as_bytes());
            input.clear();
            continue;
        } else if (DEL_STRING.to_string() == input) {
            // 検証用ファイルを削除
            if (remove_target_file(&validate_file_path) != true)  {
                panic!("Failed to remove the file named {} which you selected .");
            }
            // 実行用ファイルを削除
            if (remove_target_file(&validate_file_path) != true)  {
                panic!("Failed to remove the file named {} which you selected .");
            }
            // 再度検証用ファイルを作成する
            validate_file = create_new_file(&validate_file_path);
            validate_file.write_all("<?php \r\n".as_bytes()).unwrap();

            execute_file = create_new_file(&execute_file_path);
            execute_file.write_all("<?php \r\n".as_bytes()).unwrap();
            input.clear();
            continue;
        } else {
            // 有効なコマンドとして評価する場合
        }

        // コマンドライン用の処理を通過後再度改行文字を付与する
        input.push_str("\n");

        let target_index : i32 = input.len() as i32 - 1;
        if str_position(&input, '\\' as u8) == target_index {
            // continue;
        }


        if (input.len() == 0) {
            continue;
        }
        validate_file.write_all(input.as_bytes());



        // 以下よりphpコマンドの実行
        let mut output_result : Result<Output, Error>;
        let mut output : Output;
        if cfg!(windows) {
            output_result = Command::new(&command).args(&[&validate_file_path]).output();
            if (output_result.is_ok() != true) {
                panic!("{}", output_result.unwrap_err().to_string());
            }
        } else {
            panic!("Your machine is unsupported.");
        }

        output = output_result.unwrap();
        let exit_code : Option<i32> = output.status.code();
        let mut for_output : Vec<u8> = Vec::new();
        // コマンドの実行結果が 「0」かどうかを検証
        if (exit_code.is_some() == true && exit_code.unwrap() == 0) {

            // 検証用ファイルでプログラムが正常終了した場合
            let mut temp_file : String = fs::read_to_string(&validate_file_path).unwrap();

            // プログラムが正常終了している場合のみ改行出力を追加
            temp_file.push_str("\nprint(\"\n\"); \n");
            remove_target_file(&execute_file_path);
            execute_file = create_new_file(&execute_file_path);
            written_bytes = execute_file.write_all(temp_file.as_bytes());
            if (written_bytes.is_ok() != true) {
                panic!("Err: {}", written_bytes.unwrap_err().to_string());
            }

            // 検証用ファイルを再度削除し、改行出力を追加した分を再度保存
            remove_target_file(&validate_file_path);
            validate_file = create_new_file(&validate_file_path);
            written_bytes = validate_file.write_all(temp_file.as_bytes());

            // 実行用ファイルで再度コマンド実行
            output_result = Command::new(&command).args(&[&execute_file_path]).output();
            if (output_result.is_ok() != true) {
                panic!("{}", output_result.unwrap_err().to_string());
            }
            let _output : Vec<u8> = output_result.unwrap().stdout;
            let mut for_output : Vec<u8> = Vec::new();
            let _enum = _output.iter().enumerate();
            for (index, value) in _enum {
                // 前回まで出力した分は破棄する
                if (previous_newline_count <= current_newline_count) {
                    for_output.push(*value);
                }
                if (*value == 10) {
                    current_newline_count = current_newline_count + 1;
                }

            }
            for_output.push(10);
            let executed_result: Result <String, FromUtf8Error> = String::from_utf8(for_output.clone());
            if (executed_result.is_ok() == true) {
                println!("{}", executed_result.unwrap());
            } else {
                print_c_string(for_output);
            }
            previous_newline_count = current_newline_count;
            current_newline_count = 0;
        } else {
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


/// 引数に渡した、ファイルパスで新規ファイルを作成する
/// ファイルの作成に失敗した場合は、パニックをおこす
fn create_new_file (path : &String ) -> std::fs::File
{
    let new_file : Result <std::fs::File, Error> = fs::File::create(path);

    if (new_file.is_ok() != true) {
        // パニックでプロセスを落とす
        println!("Err {}", new_file.unwrap_err());
        panic!("Failed new file named {}", path);
    }
    return new_file.unwrap();
}


/// 引数に渡したファイルパスが存在していれば、削除。
/// できなければパニックを起こす。
fn remove_target_file (path : &String) -> bool
{
    use std::path::Path;
    // Pathオブジェクトを作成
    let target_path : &Path = Path :: new (path);
    // ファイルが存在しない場合、panicを起こす
    if (target_path.exists() != true) {
        panic!("Failed to remove the file named {} which you selected.", path);
    }
    return true;
}

/// テキストに任意の文字列が含まれているかどうか
/// 含まれない場合 -1の負の数を返却する
fn str_position (target : &String, needle : u8) -> i32
{
    let temp = target.as_bytes().iter().enumerate();
    check_type(&temp);
    for (index, value) in target.as_bytes().iter().enumerate() {
        if (needle == *value) {
            return (index as i32);
        }
    }
    return -1;
}


/// 末尾の改行文字を削除
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

/// オブジェクトの型チェック
fn check_type<T>(_: &T) -> String {
    // println!("{}", std::any::type_name::<T>());
    return std::any::type_name::<T>().to_string();
}
