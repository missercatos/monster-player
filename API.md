# Monster Siren API 文档

Base URL: `https://monster-siren.hypergryph.com`

## 响应格式

所有接口统一响应格式：

```json
{
  "code": 0,    // 0 = 成功, 104 = 未找到
  "msg": "",
  "data": {}
}
```

---

## 1. 专辑列表

```
GET /api/albums
```

返回所有专辑（全量，无分页）。

**响应 data:**

| 字段 | 类型 | 说明 |
|------|------|------|
| cid | string | 专辑 ID |
| name | string | 专辑名称 |
| coverUrl | string | 封面图片 URL |
| artistes | string[] | 艺术家列表 |

---

## 2. 专辑详情

```
GET /api/album/{cid}/detail
```

**响应 data:**

| 字段 | 类型 | 说明 |
|------|------|------|
| cid | string | 专辑 ID |
| name | string | 专辑名称 |
| intro | string | 专辑简介 |
| belong | string | 所属 (arknights) |
| coverUrl | string | 封面图片 URL |
| coverDeUrl | string | 装饰封面 URL |
| songs | array | 歌曲列表 |

**songs 子字段:**

| 字段 | 类型 | 说明 |
|------|------|------|
| cid | string | 歌曲 ID |
| name | string | 歌曲名称 |
| artistes | string[] | 艺术家列表 |

---

## 3. 歌曲列表

```
GET /api/songs
```

返回所有歌曲（全量，无分页）。

**响应 data.list:**

| 字段 | 类型 | 说明 |
|------|------|------|
| cid | string | 歌曲 ID |
| name | string | 歌曲名称 |
| albumCid | string | 所属专辑 ID |
| artists | string[] | 艺术家列表 |

---

## 4. 歌曲详情

```
GET /api/song/{cid}
```

**响应 data:**

| 字段 | 类型 | 说明 |
|------|------|------|
| cid | string | 歌曲 ID |
| name | string | 歌曲名称 |
| albumCid | string | 所属专辑 ID |
| sourceUrl | string | 音频文件直链 (.wav) |
| lyricUrl | string\|null | 歌词文件链接 |
| mvUrl | string\|null | MV 链接 |
| mvCoverUrl | string\|null | MV 封面链接 |
| artists | string[] | 艺术家列表 |

---

## 5. 新闻动态

```
GET /api/news
```

**响应 data.list:**

| 字段 | 类型 | 说明 |
|------|------|------|
| cid | string | 新闻 ID |
| title | string | 标题 |
| cate | number | 分类 |
| date | string | 日期 |
| end | boolean | 是否已到末尾 |

---

## 6. 搜索

```
GET /api/search?keyword={keyword}
```

**响应 data:**

| 字段 | 类型 | 说明 |
|------|------|------|
| albums | object | 专辑搜索结果 { list, end } |
| news | object | 新闻搜索结果 { list, end } |

---

## 音频 URL 格式

歌曲资源文件托管在 CDN，格式为：

```
https://res01.hycdn.cn/{hash1}/{hash2}/siren/audio/{date}/{hash3}.wav
```

图片资源：

```
https://web.hycdn.cn/siren/pic/{date}/{hash}.{png|jpg}
```
