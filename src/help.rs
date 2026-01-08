use crate::config::Config;
use crate::err::RpErr;
use crate::input::Input;
use crate::op::Op;
use crate::output::Output;

pub(crate) fn print_version() {
    println!("rp (rust pipe) - v0.1.0");
}

pub(crate) fn print_help(topic: Option<String>) {
    match topic {
        Some(topic) => {
            print_general_help();
            let topic = topic.to_ascii_lowercase();
            match topic.as_str() {
                "options" => print_options_help(),
                "input" => print_input_help(),
                "op" => print_op_help(),
                "output" => print_output_help(),
                "code" => print_code_help(),
                _ => (),
            }
        }
        None => print_all_help(),
    }
}

fn print_all_help() {
    print_general_help();
    print_options_help();
    print_input_help();
    print_op_help();
    print_output_help();
    print_code_help();
}

fn print_general_help() {
    print_version();
    println!("\nrp [<options>] [<input_cmd>] [<op_cmd>] [...] [<output_cmd>]");
}

fn print_options_help() {
    println!("\n<options> 选项：");
    for (_, help) in Config::all_help() {
        println!("{}", help);
    }
}

fn print_input_help() {
    println!("\n<input_cmd> 数据输入命令：");
    for (_, help) in Input::all_help() {
        println!("{}", help);
    }
}

fn print_op_help() {
    println!("\n<op_cmd> 数据操作命令：");
    for (_, help) in Op::all_help() {
        println!("{}", help);
    }
}

fn print_output_help() {
    println!("\n<output_cmd> 数据输出命令：");
    for (_, help) in Output::all_help() {
        println!("{}", help);
    }
}

fn print_code_help() {
    println!("\n命令退出码：");
    for (_, help) in RpErr::all_help() {
        println!("{}", help);
    }
}
