[![Build Status](https://app.travis-ci.com/traitmeta/soler.svg?branch=main)](https://app.travis-ci.com/traitmeta/soler)
![Tests](https://github.com/traitmeta/soler/workflows/Tests/badge.svg)
![Lints](https://github.com/traitmeta/soler/workflows/Lints/badge.svg)

# Soler

The explorer for block;

## Generate Entity

1. It should have the schema in `DATABASE_URL`
2. the use this command to generate entity `sea-orm-cli generate entity -u $DATABASE_URL -o entities/src`

## Config file

1. DB use PostgreSQL
2. API use AUXM

## Project dir

all core work is in cates

1. `scanner` is indexer for block info
2. `entities` is table of DB model and use sea-orm
3. `repo` is dal for upsert and query db
4. `api` is api with auxm
5. `config` is common configuration
