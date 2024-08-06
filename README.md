一个自动commit 工具，让你的github 账号 活跃度更高
```shell
 ✘  ~/workspace/rust/autoCommit cg run -- --help                                        
Usage: auto_commit [OPTIONS]

Options:
      --dir <dir>    需要提交的项目路径 [default: .]
      --min <min>    一天中的最小commit数量 [default: 10]
      --max <max>    一天中的最大commit数量 [default: 25]
      --cron <cron>  定时执行的cron表达式
      --m <m>        commit -m 指定提交信息 [default: "commit for update"]
      --p <p> <p>    (支持多日期提交)提交commit开始时间,结束时间,默认当天
  -h, --help         Print help
```
