# 搜索与听歌打点（给 neteasy_flutter 用）

验证程度和前两份文档一样：**返回结构**是在 GitHub Actions（境外、游客身份）对真实服务器调用得到的，并且和上游 api-enhanced 的 Node 版在同一台机器上对比过，结果一致；
**请求参数**由单元测试解密核对。没有登录态的部分会明确标出来。

## 1. 听歌打点 `/scrobble`

播放时上报，网易云才会把这首歌记进「最近播放」「听歌排行」和累计听歌数据（「我的」页的 `974 小时`、`累计听歌 8233 首` 都来自服务器的记录，不上报就不会涨）。

```text
GET /scrobble?id=<歌曲id>&sourceid=<来源id>&time=<实际播放的秒数>
```

| 参数 | 说明 |
|---|---|
| `id` | 歌曲 id |
| `sourceid` | 来源的 id：从歌单播放就传歌单 id，从专辑播放就传专辑 id；没有来源可以不传 |
| `time` | **实际播放了多少秒**（不是歌曲总时长） |

**什么时候调：每首歌调一次，在这首歌播放结束或被切走的时候，带上这次实际播放的秒数。**
一次请求里后端会依次发出两条上报：先 `startplay`（让歌曲进「最近播放」），再 `play`（带上 `time`，涨听歌计数），
所以不要在歌曲开始时调一次、结束时再调一次，那样同一首歌会被记两遍。

返回：

```json
{ "code": 200, "data": "success",
  "details": { "startplay": { "code": 200, "data": "success" }, "play": { "code": 200, "data": "success" } } }
```

已验证：游客身份下，服务器对两条上报都返回 `success`，和上游完全一致。
**没有验证的**：服务器是否真的把它计入了听歌数据，因为这需要登录账号。你可以这样验证：
登录后播一首歌并上报，过一会儿再看 `/user/detail/new` 的 `listenSongs` 和 `/listen/data/total` 的 `totalDuration` 有没有增加
（统计可能有延迟，我不知道多久）。

几点说明（都来自上游的实现，没有改）：
- 后端会用 `os=osx` 的身份和 `clientlog.music.163.com` 域名发这两条请求，这是上游验证过能被服务器接受的组合；
- `source` 固定写成 `list`，不管你实际是从歌单、每日推荐还是搜索播放的；
- 上游不做任何「播放多久才算一次」的判断，是否上报由 Flutter 决定；官方客户端的阈值我没法确认。

## 2. 搜索

### `/cloudsearch`（搜索结果页用这个）

```text
GET /cloudsearch?keywords=<关键词>&type=<类型>&limit=<条数>&offset=<偏移>
```

`type`：`1` 单曲（默认）、`10` 专辑、`100` 歌手、`1000` 歌单、`1002` 用户、`1004` MV、`1006` 歌词、`1009` 播客、`1014` 视频。
`limit` 默认 30，`offset` 默认 0。

已验证的返回结构（`result` 下的字段随 `type` 变化）：

| `type` | `result` 里的字段 |
|---|---|
| `1` | `songs[]`、`songCount` |
| `1000` | `playlists[]`、`playlistCount`、`searchQcReminder` |
| `100` | `artists[]`、`artistCount`、`searchQcReminder` |

单曲 `songs[]` 的每一项使用新版字段：`id`、`name`、`ar[]`（歌手）、`al`（专辑）、`dt`（时长，毫秒）、`fee`、`copyright`、`alia`（别名）等。
其他类型（专辑、用户、MV 等）我没有逐个看过返回，需要时把返回发给我对一下。

> 现有的 `/search` 是更老的接口，字段更少；搜索页建议统一用 `/cloudsearch`。

### `/search/default`：搜索框里的默认搜索词

返回 `data` 里的 `showKeyword`（显示在搜索框里的文案，可能带表情和前缀，服务器每次请求可能换一条）和
`realkeyword`（用户点进去后真正去搜的词），另外还有 `action`、`imageUrl`、`styleKeyword` 等。

### `/search/hot/detail`：热搜榜

`data[]` 共 20 条，每一项：`searchWord`（词）、`score`（热度）、`content`（一句说明）、`iconType`、`iconUrl`、`source`、`url`。
比 `/search/hot` 的信息多。

### `/search/multimatch`：综合匹配

```text
GET /search/multimatch?keywords=<关键词>
```

`result.orders` 是命中的卡片类型列表，比如搜「周杰伦」返回 `["artist"]`，对应 `result.artist`；
搜普通歌名时 `orders` 为空，说明没有综合卡片，直接显示结果列表即可。

### 已有的搜索相关接口

`/search/suggest?keywords=`（输入时的联想）和 `/search/hot`（简版热搜）保持不变。

## 3. 缓存

`cloudsearch`、`search/default`、`search/hot/detail`、`search/multimatch` 都是读接口，后端缓存 2 分钟（按参数区分）。
`/scrobble` 不缓存。
