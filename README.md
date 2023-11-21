# Nature Remo cloud API を使って室温のデータを取得しSQLiteに出力する
## 準備
Nature Remo Cloud APIのTokenを取得する
Developer Site : https://developer.nature.global/
Token 作成 : https://home.nature.global/

~/.remo/token.yml
にtoken を記述
```
remo: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
openweather: xxxxxxxxxxxxxxxxxxxxxxxx
```
tableを作成する
```
CREATE TABLE temp(id integer primary key autoincrement,temp float,stored text,measured text);
```

## 実行
sqlite fileを指定して実行
./remo -d temperature.sqlite

## Option
```
Options:
  -d, --db-path <DB_PATH>        sqlite database file path
  -t, --token-path <TOKEN_PATH>  remo api token file(YAML) [default: ~/.remo/token.yml]
  -h, --help                     Print help
  -V, --version                  Print version
```

