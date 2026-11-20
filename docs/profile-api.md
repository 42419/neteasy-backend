# 「我的」页接口对照（给 neteasy_flutter 用）

依据官方客户端「我的」页截图（头像区、快捷入口、音乐/播客/笔记三个标签）整理。
和 [首页文档](home-api.md) 一样，先说明验证程度，免得把猜测当成结论：

- **联网验证**：在 GitHub Actions（境外、游客身份）对真实网易云服务器调用，和上游 api-enhanced 的 Node 版在同一台机器上对比过，
  返回完全一致。游客下需要登录的接口返回 `301 需要登录`，这同样证明路由和加密是对的，但**拿不到登录后的字段**。
- **只核对请求**：单元测试里解密了后端发出的请求，路径、加密方式、参数与上游一致。
- **登录后的字段**：用你在已登录状态下导出的真实 JSON 核对过（见第 1 节）；仍然标「待确认」的，是还没有对应数据的部分。

> 所有个人数据接口**不进后端缓存**（点赞、收藏、关注后应该立刻刷新）；游客是否能拿到数据，取决于后端有没有成功拿到游客 token。

## 1. 截图里每个位置用哪个接口

| 截图位置 | 接口 | 状态 |
|---|---|---|
| 头像、昵称、性别、个性签名 | `/user/detail/new?uid=` 或已有的 `/user/detail?uid=`；当前账号 uid 用 `/user/account`。字段：`profile.avatarUrl`、`profile.nickname`、`profile.gender`（2 是 ♀，1 是 ♂）、`profile.signature` | **已核对**（与截图一致） |
| `SVIP·伍` 徽章 | `/vip/info/v2`：等级 `data.redVipLevel`（5 → 「伍」）；`data.redplus`（黑胶 SVIP）的 `expireTime` 大于 `data.now` 就是 SVIP 在有效期内；`data.associator` 是黑胶 VIP | 等级**已核对**；「用 redplus 判断是不是 SVIP」是我的推断，没有未开通 SVIP 的账号可对照 |
| `18 枚徽章` | `/user/medal`：数 `data.obtainMedals` 里 `canWear == true` 的个数。注意 `data.medalNum` 是 20（含两枚不能佩戴的会员徽章），**不是**页面上显示的数字 | **已核对**（恰好 18） |
| `3 关注`、`1 粉丝` | 数量：`/user/detail/new` 的 `profile.follows`、`profile.followeds`；列表：`/user/follows`、`/user/followeds` | 数量**已核对** |
| `Lv.9` | `/user/detail/new` 的 `level`，或 `/user/level` 的 `data.level`（还有升级进度 `data.progress`、`nowPlayCount / nextPlayCount`、`nowLoginCount / nextLoginCount`、权益说明 `data.info`，用 `$` 分隔） | **已核对** |
| `974 小时` | `/listen/data/total` 的 `data.totalDuration`，**单位是秒**，`÷ 3600` 向下取整 | **已核对**（3508051 秒 → 974） |
| 快捷入口「最近」 | `/record/recent/song`、`/playlist`、`/album`、`/video`、`/voice`、`/dj`。`/recent/listen/list` 返回的是「最近常听」（歌单 / 专辑 / 榜单混合，带 `tag`、`landingUrl`），**不是**「我的」页里的 `近期` 标签 | 联网通；`recent/listen/list` 的结构**已核对** |
| 快捷入口「本地」 | 客户端本地功能，没有接口 | — |
| 快捷入口「网盘」 | `/user/cloud`（列表）、`/user/cloud/detail?id=1,2`（详情） | 需要登录 |
| 快捷入口「云贝」 | `/yunbei/info`、`/yunbei/tasks`、`/yunbei/tasks/todo`、`/yunbei/task/list/v1`；是否已签到 `/yunbei`、今日收支 `/yunbei/today` | `info`/`tasks` 联网通；后两个需要登录 |
| 标签「播客」 | `/user/dj`（我创建的节目）、`/dj/sublist`（收藏的播客）、`/record/recent/dj` | 联网通 |
| 标签「笔记」 | **来源未知**。你反馈没有 `mlog/mycollect` 这个接口，说明我猜错了。可选的上游模块只有 `mlog_*`、`user_event`、`playlist_video_recent` 几个，都不像「笔记」。需要抓一次官方客户端的请求才能确定（见第 4 节） | 待确认 |
| 音乐 · `创建²` `收藏²` 的数字 | `/user/subcount` 的 `createdPlaylistCount`（含「我喜欢的音乐」）、`subPlaylistCount`；另有 `djRadioCount`（收藏的播客）、`artistCount`、`mvCount`、`programCount` | **已核对**（2 和 2） |
| 音乐 · 歌单列表 | 已有的 `/user/playlist?uid=`：`creator.userId` 等于自己的是「创建」，否则是「收藏」；`specialType == 5` 是「我喜欢的音乐」。收藏的歌单还有 `/user/playlist/collect?uid=` | 联网通 |
| 音乐 · `近期` 子标签 | 内容是**自己创建 + 收藏的歌单**（最多 30 条）再加一个「听歌排行」入口，不含「最近常听」里那些没收藏的歌单；排序依据待确认，需要对照 `/user/playlist` 的返回 | 待确认 |
| 「我喜欢的音乐」`110 首 · 2224 次播放` | `/user/playlist` 里该歌单的 `trackCount`、`playCount` | 已有接口 |
| `心动模式` 按钮 | `/playmode/intelligence/list?id=<当前歌曲id>&pid=<我喜欢的音乐歌单id>` | 联网验证：用非「我喜欢」歌单会返回 `400 不支持该歌单类型`，所以 `pid` 必须是「我喜欢的音乐」 |
| 「听歌排行」`累计听歌 8233 首` | 数量：`/user/detail/new` 的 `listenSongs`；`🔒`：`peopleCanSeeMyPlayRecord == false`；排行列表：`/user/record?uid=&type=0`（只能看自己的，别人返回 `-2 无权限`） | 数量和锁**已核对** |
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

## 4. 页面数字的取法（已用真实数据核对）

```dart
// 974 小时：累计听歌时长，单位是秒
final hours = (listenTotal['data']['totalDuration'] as int) ~/ 3600;

// 18 枚徽章：能佩戴的已获得徽章数（data.medalNum 是 20，包含两枚不能佩戴的会员徽章）
final badges = (medal['data']['obtainMedals'] as List)
    .where((m) => m['canWear'] == true)
    .length;

// SVIP·伍：等级取 redVipLevel；SVIP 是否在有效期内看 redplus
final vip = vipInfoV2['data'];
final svipActive = (vip['redplus']?['expireTime'] ?? 0) > vip['now'];
const numerals = ['', '壹', '贰', '叁', '肆', '伍', '陆', '柒', '捌', '玖', '拾'];
final badgeText = 'SVIP·${numerals[vip['redVipLevel'] as int]}';

// 听歌排行右边的小锁：自己的听歌记录不对别人公开
final recordPrivate = detail['peopleCanSeeMyPlayRecord'] == false;
```

## 5. 还没有确认的

1. **「笔记」标签的数据来源**：`mlog/mycollect` 不对。请告诉我你当时看到的是什么：路由 404、返回空数据，还是返回需要登录？
   如果上游里没有对应接口，就只能抓一次官方客户端「我的 → 笔记」标签的请求（用 mitmproxy、HttpCanary 之类，
   只要请求路径、参数名和返回体的前几项，**不要带 cookie**），我再按抓到的接口移植。
2. **`近期` 标签的排序**：需要 `/user/playlist?uid=<你的uid>` 的返回，对照 `updateTime`、`specialType`、`subscribed` 和你截图里的顺序。
3. **右上角宫格图标**里面有哪些入口：点开截一张图给我。
4. **`SVIP` 与 `VIP` 徽章的区分规则**：目前只是推断，没有未开通 SVIP 的账号可以对照。
