# 「我的」页接口对照（给 neteasy_flutter 用）

依据官方客户端「我的」页截图（头像区、快捷入口、音乐/播客/笔记三个标签）整理。
和 [首页文档](home-api.md) 一样，先说明验证程度，免得把猜测当成结论：

- **联网验证**：在 GitHub Actions（境外、游客身份）对真实网易云服务器调用，和上游 api-enhanced 的 Node 版在同一台机器上对比过，
  返回完全一致。游客下需要登录的接口返回 `301 需要登录`，这同样证明路由和加密是对的，但**拿不到登录后的字段**。
- **只核对请求**：单元测试里解密了后端发出的请求，路径、加密方式、参数与上游一致。
- **登录后的字段含义我没法验证**。下面标「待确认」的，需要你在已登录状态下把对应接口的 JSON 发给我。

> 所有个人数据接口**不进后端缓存**（点赞、收藏、关注后应该立刻刷新）；游客是否能拿到数据，取决于后端有没有成功拿到游客 token。

## 1. 截图里每个位置用哪个接口

| 截图位置 | 接口 | 状态 |
|---|---|---|
| 头像、昵称、性别、个性签名 | `/user/detail/new?uid=` 或已有的 `/user/detail?uid=`；当前账号 uid 用 `/user/account` | 联网通（游客）；字段含义待确认 |
| `SVIP·伍` 徽章 | `/vip/info/v2?uid=`、`/vip/info?uid=` | 联网通，游客态没有会员数据；**等级字段待确认** |
| `18 枚徽章` | `/user/medal?uid=` | 联网通；待确认 |
| `3 关注`、`1 粉丝` | 数量：`/user/detail/new` 的 `profile.follows / followeds`（待确认）；列表：`/user/follows`、`/user/followeds` | 列表联网通 |
| `Lv.9` | `/user/level` | 需要登录；字段待确认 |
| `974 小时` | 很可能是 `/listen/data/total`（累计听歌时长），**我没法确认** | 需要登录；待确认 |
| 快捷入口「最近」 | `/record/recent/song`、`/playlist`、`/album`、`/video`、`/voice`、`/dj`；汇总：`/recent/listen/list` | 联网通 |
| 快捷入口「本地」 | 客户端本地功能，没有接口 | — |
| 快捷入口「网盘」 | `/user/cloud`（列表）、`/user/cloud/detail?id=1,2`（详情） | 需要登录 |
| 快捷入口「云贝」 | `/yunbei/info`、`/yunbei/tasks`、`/yunbei/tasks/todo`、`/yunbei/task/list/v1`；是否已签到 `/yunbei`、今日收支 `/yunbei/today` | `info`/`tasks` 联网通；后两个需要登录 |
| 标签「播客」 | `/user/dj`（我创建的节目）、`/dj/sublist`（收藏的播客）、`/record/recent/dj` | 联网通 |
| 标签「笔记」 | 可能是 `/mlog/mycollect`（xeapi），**我是猜的** | 需要登录；待确认 |
| 音乐 · `创建²` `收藏²` 的数字 | `/user/subcount`（`createdPlaylistCount` 等） | 联网通 |
| 音乐 · 歌单列表 | 已有的 `/user/playlist?uid=`：`creator.userId` 等于自己的是「创建」，否则是「收藏」；`specialType == 5` 是「我喜欢的音乐」。收藏的歌单还有 `/user/playlist/collect?uid=` | 联网通 |
| 音乐 · `近期` 子标签 | 排序依据我没法确认，可能是 `/recent/listen/list`；底部写着「展示创建/收藏的 30 条内容」 | 待确认 |
| 「我喜欢的音乐」`110 首 · 2224 次播放` | `/user/playlist` 里该歌单的 `trackCount`、`playCount` | 已有接口 |
| `心动模式` 按钮 | `/playmode/intelligence/list?id=<当前歌曲id>&pid=<我喜欢的音乐歌单id>` | 联网验证：用非「我喜欢」歌单会返回 `400 不支持该歌单类型`，所以 `pid` 必须是「我喜欢的音乐」 |
| 「听歌排行」`累计听歌 8233 首` | 数量：`profile.listenSongs`（待确认）；排行列表：`/user/record?uid=&type=0`（只能看自己的，别人返回 `-2 无权限`） | 联网验证了权限行为 |
| 顶部「添加状态」 | 读取：`/user/social/status?uid=`；推荐状态：`/user/social/status/rcmd` | 前者联网通 |
| 顶部搜索 | 已有 `/search` 等 | — |
| `新建歌单`、`导入外部歌单`、`置顶` | **写操作，没有移植** | — |

## 2. 会员相关（不止徽章）

| 接口 | 用途 |
|---|---|
| `/vip/info`、`/vip/info/v2` | 会员状态、类型、到期时间（两个版本字段不同） |
| `/vip/growthpoint`、`/vip/growthpoint/details` | 成长值概览 / 明细 |
| `/vip/tasks`、`/vip/tasks/v1?id=` | 会员任务（只读，不领取） |
| `/vip/timemachine` | 黑胶时光机（`startTime`、`endTime` 都传了才会带时间范围） |
| `/vip/sign/info`、`/vip/sign/detail`、`/vip/sign/history`、`/signin/progress` | 会员签到信息（只读，不会签到） |
| `/listen/data/total`、`/today/song`、`/year/report`、`/report`、`/realtime/report`、`/song/play/rank` | 听歌数据与报告（登录后可用） |

## 3. 没有移植的接口

会改动账号状态的都没加：签到、领取成长值 / 云贝、关注 / 取关、修改资料、新建 / 导入 / 排序歌单、删除云盘文件、退出登录、绑定 / 换绑手机等。
这类操作不是「信息」，自动签到、领取也容易触发风控；需要时告诉我再加。

另外 `yunbei/expense`、`yunbei/receipt` 上游有，但真实服务器返回 `404 接口未找到`（上游 Node 版同样如此），说明网易云已经下线，所以没有移植。

## 4. 请你发给我的 JSON（已登录状态）

把下面这些接口的返回发给我（**不要带 cookie**，返回体里如果有手机号、邮箱之类的个人信息请先打码），
我就能把上面「待确认」的字段逐个对准，也能判断还缺哪些接口：

1. `/user/detail/new?uid=<你的uid>`：头像、昵称、性别、关注 / 粉丝数、听歌数
2. `/user/level`：`Lv.9` 和 `974 小时` 各来自哪个字段
3. `/vip/info/v2?uid=<你的uid>`：`SVIP·伍` 的等级字段
4. `/listen/data/total`：累计时长（`974 小时` 是不是它）
5. `/user/subcount`：`创建²`、`收藏²` 的数字
6. `/user/medal?uid=<你的uid>`：`18 枚徽章`
7. `/recent/listen/list`：看「近期」是不是它
8. `/mlog/mycollect`：看「笔记」是不是它

只发每个接口返回里的前几项就行，不需要完整数据。
