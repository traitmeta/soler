[![Build Status](https://app.travis-ci.com/traitmeta/soler.svg?branch=main)](https://app.travis-ci.com/traitmeta/soler)

# soler

The explorer for block;

## 生成 Entity

1. 首先需要有数据库表
2. 然后使用命令`sea-orm-cli generate entity -u $DATABASE_URL -o entities/src`

## Config file

1. DB use PostgreSQL
2. API use AUXM

## Project dir

1. `scanner` is indexer for block info
2. `entities` is table of DB model and use sea-orm
3. `repo` is dal for upsert and query db
4. `core` is api with auxm
5. `config` is common configuration


