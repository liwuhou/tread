---
command: alive-systems
description: 显示直播后台系统清单并检查本地环境
---

# Alive Systems 命令

## /alive-systems

显示直播后台系统清单，并检查本地环境。

**触发关键词**：

- "直播后台系统"
- "alive systems"
- "系统列表"
- "检查后台环境"

---

## /clone-system

克隆指定的直播后台系统到本地。

**触发关键词**：

- "clone {系统名}"
- "下载 {系统名}"
- "拉取 {系统名} 代码"

---

## /find-api

在已克隆的系统中搜索相关接口。

**触发关键词**：

- "查找 {业务关键词} 接口"
- "哪里有用到 {功能}"
- "参照哪个接口"

---

## 使用示例

```
/alive-systems
  -> 显示系统清单，询问/读取本地目录，检查环境

/clone-system abs-go
  -> 从系统清单获取地址，git clone 到本地

/find-api 拉流
  -> 在 abs-go 中搜索拉流相关接口代码
```
