[![Build Status](https://app.travis-ci.com/traitmeta/soler.svg?branch=main)](https://app.travis-ci.com/traitmeta/soler)

# soler

this repo include something about rust.

## 生成Entity

1. 首先需要有数据库表
2. 然后使用命令`sea-orm-cli generate entity -u $DATABASE_URL -o entities/src`

## TODO

1. ~~auxm 换成最新版本~~
2. sea-orm使用GraphQL
3. Auxm使用seaOrm
4. ~~sui 追踪指定package中的event~~
5. ~~RPC main拆分成独立的测试代码~~
6. kafka producer 通用改造
7. sui 指定package中的event 分页遍历
