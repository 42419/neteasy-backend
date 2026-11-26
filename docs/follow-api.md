# 「关注」页接口（给 neteasy_flutter 用）

> **先说明一个局限**：我没有看到官方「关注」页的截图或你的设计，下面是按上游接口的能力整理的。
> 动态流 `/event` 需要登录才有内容，我只能用**公开用户的动态**（`/user/event`）核对条目结构；两者的条目结构相同，
> 但登录后你自己的关注流里实际出现哪些类型，我没法验证。你把官方「关注」页（以及点进一条动态）的截图发我，我再逐项对。

验证方式和前几份文档一样：返回结构来自 GitHub Actions（境外、游客身份）对真实服务器的调用，并和上游 Node 版在同一台机器上对比过；
请求参数由单元测试解密核对。

## 1. 动态流

### `GET /event`：关注的人的动态（需要登录）

| 参数 | 说明 |
|---|---|
| `pagesize` | 每页条数，默认 20 |
| `lasttime` | 翻页游标：把上一页返回的 `lasttime` 原样带回来；第一页不传（默认 -1） |

### `GET /user/event?uid=<用户id>`：某个用户发布的动态（公开）

参数 `uid`、`limit`（默认 30）、`lasttime`（翻页，同上）。**已用真实数据验证**。

两个接口的返回结构相同：

```text
{ code, events[], lasttime, more, size }      more 为 true 表示还有下一页，下一页带上 lasttime
```

### 动态条目 `events[i]`（已验证的字段）

| 字段 | 说明 |
|---|---|
| `id`、`uuid` | 动态 id |
| `type` | 动态类型。**我只见到 `18`（分享单曲）**，其他类型没有见过，不要猜它们的含义 |
| `eventTime` | 发布时间，毫秒时间戳 |
| `user` | 发布者：`userId`、`nickname`、`avatarUrl` 等 |
| **`json`** | **是一个 JSON 字符串，要再解析一次**（见下面的示例） |
| `info` | 互动信息：`liked`（我是否点赞）、`likedCount`、`commentCount`、`shareCount`、`threadId`、`commentThread`、`latestLikedUsers`、`comments` |
| `pics[]` | 配图 |
| `forwardCount`、`topEvent`、`expireTime` | 转发数、是否置顶、过期时间 |

`type == 18` 时，解析 `json` 得到：`msg`（发布者写的话）、`song`（歌曲：`id`、`name`、`artists`、`album`……）、`title`、`songEvaluation`、`soundeffectsInfo`。

```dart
import 'dart:convert';

for (final e in res['events'] as List) {
  final payload = jsonDecode(e['json'] as String) as Map<String, dynamic>;
  final text = payload['msg'];                 // 发布者写的话
  final song = payload['song'];                // type == 18 时才有
  final threadId = e['info']['threadId'];      // 评论、点赞都用它，形如 A_EV_2_<动态id>_<用户id>
}
```

## 2. 动态的互动

| 功能 | 接口 | 说明 |
|---|---|---|
| 看评论 | `GET /comment/event?threadId=<info.threadId>` | 参数 `limit`、`offset`、`before`；用一个不存在的 id 调用，返回的是正常的空评论结构 |
| 评论点赞 | 已有的 `GET /comment/like?type=6&threadId=<threadId>&cid=<评论id>&t=1` | `t=1` 点赞，其他值取消 |
| 动态点赞 | `GET /resource/like?type=6&threadId=<info.threadId>&t=1` | **写操作**；`t=1` 点赞，其他值取消。点赞动态必须带 `type=6` 和 `threadId` |
| 搜索动态 | `GET /event/search?keywords=` | 联网通；返回 `data`、`message`；没有结果时的结构我没细看 |

## 3. 关注关系

| 功能 | 接口 | 说明 |
|---|---|---|
| 关注 / 取消关注 | `GET /follow?id=<对方用户id>&t=1` | **写操作**；`t=1` 关注，其他值（或不传）取消关注 |
| 关注列表 | `GET /user/follows?uid=` | 公开用户已验证：`follow[]`、`more` |
| 粉丝列表 | `GET /user/followeds?uid=` | 本次未单独联网验证（上一轮已验证过路由和加密） |
| 关注列表（用户和艺人混合） | `GET /user/follow/mixed?scene=` | 需要登录，游客返回 `400 not login`（上游同样）；`scene`：0 全部、1 关注的歌手、2 关注的用户；翻页用 `size`、`cursor` |
| 是否互相关注 | `GET /user/mutualfollow/get?uid=` | 已有 |

用户对象里的 `followed` 字段表示「我是否已关注他」，可以用来决定按钮显示「关注」还是「已关注」。

## 4. 话题

| 接口 | 说明 |
|---|---|
| `GET /hot/topic?limit=&offset=` | 热门话题，返回 `hot[]`（有游客 token 时联网通） |
| `GET /topic/detail?actid=` | 话题详情 |
| `GET /topic/detail/event/hot?actid=` | 话题下的热门动态 |

后两个我没有找到合适的 `actid`，只核对了请求参数，没有联网验证返回结构。

## 5. 没有移植的

- **动态的写操作**：转发（`event_forward`）、删除（`event_del`）、改隐私（`event_privacy`）、分享到动态（`share_resource`，它还依赖反作弊 token）。和之前不做发评论的原因一样。
- **视频流**（`video_timeline_*`）、**私信与通知**（`msg_*`）：不确定「关注」页要不要，等你确认。

## 6. 缓存

`/hot/topic`、`/topic/detail`、`/topic/detail/event/hot` 是公共内容，缓存 2 分钟；
动态流、关注关系和两个写操作都是个人数据，**不缓存**（关注、点赞之后应该立刻刷新）。
