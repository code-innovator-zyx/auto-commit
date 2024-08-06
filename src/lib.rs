use std::{
    fs::File,
    io::{self, Write},
    process::Command,
    str::FromStr,
    thread, vec,
};

use chrono::{Duration, Local, NaiveDate};
use clap::ArgMatches;
use cron::Schedule;
use rand::Rng;

#[derive(Debug)] // 日点击量
pub struct DailyCommit {
    date: String,
    commit: usize,
}
#[derive(Debug)]
pub struct CommitHandle {
    time_arr: Vec<DailyCommit>,
    project_dir: String,
    commit_msg: String,
    cron_expression: Option<String>,
    min_commit: usize,
    max_commit: usize,
}

impl CommitHandle {
    pub fn new(args: ArgMatches) -> CommitHandle {
        let dir = args.get_one::<String>("dir").unwrap();
        let min = args
            .get_one::<String>("min")
            .unwrap_or(&String::from("0"))
            .parse::<usize>()
            .unwrap();
        let max = args
            .get_one::<String>("max")
            .unwrap_or(&String::from("0"))
            .parse::<usize>()
            .unwrap();
        let cron = args.get_one::<String>("cron");
        let mut period: Vec<_> = args.get_many::<String>("p").unwrap_or_default().collect();
        let today = Local::now().format("%Y-%m-%d").to_string();
        if period.is_empty() {
            period = vec![&today, &today];
        }
        let mut handler = CommitHandle {
            time_arr: Vec::new(),
            project_dir: dir.clone(),                    // 默认当前目录
            commit_msg: "commit for update".to_string(), // 默认commit msg
            cron_expression: cron.map(|s| s.clone()),
            min_commit: min,
            max_commit: max,
        };

        if cron.is_none() {
            // 如果不是定时任务，初始化时间段
            handler.parse_daily_time(period[0], period[1], min, max);
        }
        handler
    }
    // fn current_dir(&self) -> String {
    //     let path_buf = current_dir().unwrap_or_else(|_| PathBuf::from("."));
    //     // 使用 to_string_lossy() 生成一个 String
    //     path_buf.to_string_lossy().into_owned()
    // }
    // 解析范围日期内每一天的commit数量
    fn parse_daily_time(&mut self, start: &str, end: &str, min: usize, max: usize) {
        let mut start = NaiveDate::parse_from_str(start, "%Y-%m-%d").unwrap();
        let end = NaiveDate::parse_from_str(end, "%Y-%m-%d").unwrap();
        while start <= end {
            self.time_arr.push(DailyCommit {
                date: start.to_string(),
                commit: rand::thread_rng().gen_range(min..max),
            });
            start += Duration::days(1)
        }
    }

    // 获取一个随机时间
    fn rand_daily_time(&self, date: &str, total: usize, current_index: usize) -> String {
        // 根据当天总的提交数和当前提交批次，设置提交时间
        let mut rng = rand::thread_rng();
        // 分钟在0-59可以随机
        let minute = rng.gen_range(0..59);
        // 小时计算   需要先将总的时间分批次，可以为
        let batch_size = 12 / total;
        let extra_hours = 12 % total;

        // 当前批次的开始和结束时间
        let batch_start_hour = 8 + current_index * batch_size + extra_hours.min(current_index);
        let batch_end_hour = batch_start_hour + batch_size + (extra_hours > current_index) as usize;

        // 生成随机时间
        let mut rng = rand::thread_rng();
        let hour = rng.gen_range(batch_start_hour..batch_end_hour + 1);
        format!("{date} {:02}:{:02}", hour, minute)
    }

    // 重置提交文件
    fn reset_commit_file(&self, time_str: &str) -> io::Result<()> {
        let commit_content = format!(
            "{}\n随机数: {}",
            time_str,
            rand::thread_rng().gen_range(1..=100000),
        );
        let file_path = format!("{}/commit.md", self.project_dir);
        let mut file = File::create(file_path).expect("Unable to create file");
        file.write_all(commit_content.as_bytes())
            .expect("unable to wrtite data");
        Ok(())
    }

    // 执行终端命令
    fn execute_command(&self, cmd: &str) -> io::Result<String> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .current_dir(&self.project_dir)
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Command failed: {cmd}\n{stderr}"),
            ));
        }
        Ok(stdout)
    }

    // 执行任务
    pub fn run(&mut self) -> io::Result<()> {
        if let Some(expression) = self.cron_expression.take() {
            // 周期性定时任务
            self.cron_job(expression.as_str())?;
        } else {
            // 一次性任务
            let mut times = 0;
            for ele in &self.time_arr {
                self.once_day(ele.commit, &ele.date)?;
                times += ele.commit
            }
            self.push_commits()?;
            println!("总共成功推送 {} 个 commit", times);
        }
        Ok(())
    }

    // 后台定期执行的任务
    fn cron_job(&self, expression: &str) -> io::Result<()> {
        // let expression = "0 54 9 * * *";
        let schedule = Schedule::from_str(expression).expect("Failed to parse CRON expression");
        let times = rand::thread_rng().gen_range(self.min_commit..self.max_commit);
        println!("执行周期性的自动commit任务");
        println!("cron expression : 【{}】", expression);
        loop {
            let now = Local::now();
            if let Some(next) = schedule.upcoming(Local).take(1).next() {
                let until_next = next - now;
                println!("下一次运行时间: [{}]", next);
                thread::sleep(until_next.to_std().unwrap());
                {
                    let date = Local::now().format("%Y-%m-%d").to_string();
                    self.once_day(times, date.as_str())?;
                    self.push_commits()?;
                    println!("[{}] 成功推送 {} 个 commit", date, times);
                }
            }
        }
    }

    // 一天的commit 次数提交
    fn once_day(&self, times: usize, date_str: &str) -> io::Result<()> {
        for index in 1..times {
            let commit_time = self.rand_daily_time(date_str, times, index);
            let formatted_time = format!("{}T00:00:00Z", commit_time);
            self.commit_file(formatted_time.as_str())?;
        }
        Ok(())
    }
    // 执行commit 指令
    fn commit_file(&self, commit_time_str: &str) -> io::Result<()> {
        if let Err(e) = self.reset_commit_file(commit_time_str) {
            println!("更新文件失败,{}", e)
        }
        // 添加到暂存区
        let add_cmd = "git add .";
        let commit_cmd = format!(
            "git commit --date='{}' -am '{}'",
            commit_time_str, self.commit_msg
        );
        self.execute_command(add_cmd)?;
        self.execute_command(&commit_cmd)?;
        println!("{} 成功添加一个commit ", commit_time_str);
        Ok(())
    }

    fn push_commits(&self) -> io::Result<()> {
        let cmd = "git pull --rebase && git push";
        self.execute_command(cmd)?;
        Ok(())
    }
}
