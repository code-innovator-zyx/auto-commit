use auto_commit::CommitHandle;
use clap::{Arg, Command};
use std::{env::current_dir, path::PathBuf};

fn main() {
    // 获取当前工作目录
    let path_buf = current_dir().unwrap_or_else(|_| PathBuf::from("."));
    // 使用 to_string_lossy() 生成一个 String
    let dir = path_buf.to_string_lossy().into_owned(); // 使用 to_string_lossy() 转换为 String
    let matches = Command::new("auto-commit")
        .arg(
            Arg::new("dir")
                .long("dir")
                .help("需要提交的项目路径")
                .num_args(1)
                .default_value("."),
        ) // 使用 num_args(1) 代替 takes_value(true)
        .arg(
            Arg::new("min")
                .long("min")
                .help("一天中的最小commit数量")
                .num_args(1)
                .default_value("10"),
        ) // 指定参数个数
        .arg(
            Arg::new("max")
                .long("max")
                .help("一天中的最大commit数量")
                .num_args(1)
                .default_value("25"),
        )
        .arg(
            Arg::new("cron")
                .long("cron")
                .help("定时执行的cron表达式")
                .num_args(1),
        )
        .arg(
            Arg::new("m")
                .long("m")
                .help("commit -m 指定提交信息")
                .num_args(1)
                .default_value("commit for update"),
        ) // 指定参数个数
        .arg(
            Arg::new("p")
                .long("p")
                .help("(支持多日期提交)提交commit开始时间,结束时间,默认当天")
                .num_args(2),
        )
        .get_matches();

    let mut handler = CommitHandle::new(matches);
    if let Err(e) = handler.run() {
        println!("执行失败, {}", e)
    }
    // println!("{}", handler)
}
