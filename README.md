# Whisper
mini pub - sub framework

## Usage

### 1. Broker mode

#### 1.1 Execution
```
./whisper broker
```

### 2. Node mode

#### 2.1 Execution
```
./whisper node
```

#### 2.2 Subscribe Message

SUB <MESSAGE_NAME>

```
./whisper node
SUB chat
```


#### 2.3 Publish Message

PUB <MESSAGE_NAME> <MESSAGE_CONTENT>

```
./whisper node
PUB chat hello
```
