## How to run this code

### Firstly start python server
```
$ python3 server/buggy_server.py
```
and tou will see this:
```
Length of data: <DATA_LENGTH>
SHA-256 hash of the data: <DATA_HASH>
Starting HTTP server on port 127.0.0.1:8080
```
you will need ```<DATA_LENGTH>``` and ```<DATA_HASH>``` to run client code

### How to run Rust code

run in second terminal window in folder ```./rust\ client```
```
$ cargo run --release -- <DATA_LENGTH> <DATA_HASH>
```
and you will see how cliend and server works

### How to run Kotlin code

run in second terminal window in folder ```./kotlin\ client```
```
$ kotlinc main.kt -include-runtime -d client.jar
```
Now kotlin compiled and we can start it by
```
java -jar client.jar <DATA_LENGTH> <DATA_HASH>
```
and you will see how cliend and server works
