# 首页接口对照（给 neteasy_flutter 用）

目标：Flutter 首页 1:1 复刻网易云音乐首页。本文说明首页每个模块该用哪个接口、返回结构长什么样，以及哪些结论已经验证、哪些还没有。

> 验证环境说明：下面的「已验证」指在 GitHub Actions（境外机器、**游客身份**）上对真实网易云服务器调用的结果；
> 「只核对了请求」指单元测试里解密了后端发出的请求，确认路径、加密方式和参数与上游一致，但没有联网。
> 登录态下返回的内容更多（见第 2 节），那部分我没法替你验证。

## 1. 先看结论：首页其实是一个接口

官方客户端的首页区块（推荐歌单、云村热播、雷达歌单、有声书……）不是一个模块一个接口，而是由
**`GET /homepage/block/page`** 按官方顺序一次返回的。你在 `app_constants.dart` 里标的 6 个路由名有一半是猜的，
上游并不存在：`/personalized/artist`、`/personalized/radar`、`/voicelist/recommend`、`/dj/program/recommend`。

所以推荐的做法是：**首页主体改用 `/homepage/block/page` 渲染**，按 `blockCode` 决定用哪种组件；
下面的单独接口只在需要点进去看详情、或者区块里没有数据时作为补充。

### `/homepage/block/page` 的结构（已验证，游客）

```text
GET /homepage/block/page?refresh=false&cursor=<上一页返回的 cursor>
data
├─ blocks[]                         区块列表，顺序就是官方首页顺序
│  ├─ blockCode                     区块标识，如 HOMEPAGE_BLOCK_PLAYLIST_RCMD
│  ├─ showType                      展示样式，如 HOMEPAGE_SLIDE_PLAYLIST
│  ├─ uiElement                     区块头：mainTitle.title / subTitle.title / button{action,text}
│  └─ creatives[]                   区块里的每一项
│     ├─ creativeType               如 list / SONG_LIST_HOMEPAGE / NEW_SONG_HOMEPAGE
│     ├─ action / actionType        点击跳转，如 orpheus://playlist/<id>
│     ├─ uiElement                  标题、图片 image.imageUrl、labelTexts
│     └─ resources[]                具体资源：resourceType（list / song / ...）、resourceId、uiElement…
├─ hasMore, cursor                  翻页：hasMore 为 true 时带 cursor 再请求
├─ blockCodeOrderList, titles, pageConfig, ...
```

游客状态下实测只返回 3 个区块：

| blockCode | showType | 说明 |
|-----------|----------|------|
| `HOMEPAGE_BLOCK_PLAYLIST_RCMD` | `HOMEPAGE_SLIDE_PLAYLIST` | 推荐歌单，12 项，每项一个歌单（`resources[0].resourceType = list`） |
| `HOMEPAGE_BLOCK_STYLE_RCMD` | `HOMEPAGE_SLIDE_SONGLIST_ALIGN` | 曲风推荐，4 项，每项 3 首歌（`creativeType = SONG_LIST_HOMEPAGE`），按钮是「播放全部」 |
| `HOMEPAGE_BLOCK_NEW_ALBUM_NEW_SONG` | `HOMEPAGE_NEW_SONG_NEW_ALBUM` | 新歌新碟，6 项，每项 3 首歌 |

`/homepage/dragon/ball`（顶部圆形入口）游客状态下返回 `{"code":200,"data":[]}`，登录后是否有内容未验证。

## 2. 你标的 6 个模块怎么接

| Flutter 里的占位 | 推荐数据源 | 备选 / 说明 |
|---|---|---|
| 近期云村热播 `/program/recommend` | `/homepage/block/page` 里对应的区块 | 单独接口 `/program/recommend` 返回 `programs[]`（节目列表）。**对不对得上官方「云村热播」区块我没法确认**：游客页里没有这个区块 |
| 说唱 \| 电子 \| 摇滚 热门精选 | `HOMEPAGE_BLOCK_STYLE_RCMD`（曲风推荐，已验证有） | 也可以用标签歌单：`/top/playlist?cat=说唱&limit=…`（返回 `playlists[]`）；`/personalized/newsong` 返回 `result[]` |
| 选择你喜欢的艺人，为你定制歌单 | 登录后的区块页（待你导出区块清单确认） | 上游没有 `personalized/artist`。相关接口：`/style/preference`（曲风偏好，需登录）、`/top/artists`（`artists[]`）、`/toplist/artist`（`list`） |
| 我的雷达歌单 | 登录后的区块页（待确认） | 上游没有专门的雷达接口。雷达歌单本质是普通歌单，拿到 id 后用已有的 `/playlist/detail?id=` 取详情 |
| 根据你听过的热门节目推荐 | `/personalized/djprogram`（返回 `result[]`） | `/dj/program/toplist`（`toplist[]`）、`/dj/recommend`（`djRadios[]`）；详情用 `/dj/program/detail?id=`、`/dj/detail?rid=`、`/dj/program?rid=` |
| SVIP 专享有声书免费听 | 登录后的区块页（待确认） | 上游的 `voicelist/*` 是创作者后台接口，不是首页推荐。`/voicelist/detail?id=`、`/voicelist/list?voiceListId=` 可用于点进去 |

### 登录后导出区块清单（建议你做一次）

登录态下区块更多，我没有登录态，没法替你看。你在 Flutter 里（已登录）临时打印一次就行，**只需要这几项，不要贴 cookie**：

```dart
final res = await api.get('/homepage/block/page');            // 你现有的 ApiClient
for (final b in (res['data']['blocks'] as List)) {
  print('${b['blockCode']} | ${b['showType']} | '
      '${b['uiElement']?['mainTitle']?['title']} | n=${(b['creatives'] as List).length}');
}
print('hasMore=${res['data']['hasMore']} cursor=${res['data']['cursor']}');
```

把输出的区块清单发给我，我就能把上表里“待确认”的几项对准，并补上缺的接口。

## 3. 本次新增的接口

加密方式、路径和参数都照 api-enhanced；表里“返回的顶层字段”是真实服务器游客调用的结果，标「—」的是只核对了请求。

| 路由 | 主要参数 | 返回的顶层字段 |
|---|---|---|
| `/homepage/block/page` | `refresh`、`cursor` | `data.blocks` 等，见上文 |
| `/homepage/dragon/ball` | — | `data[]`（游客为空） |
| `/program/recommend` | `type`（分类 id）、`limit`、`offset` | `programs`、`more`、`name` |
| `/personalized/newsong` | `limit`、`areaId` | `result`、`category` |
| `/personalized/djprogram` | — | `result`、`category` |
| `/top/playlist` | `cat`、`order`（hot/new）、`limit`、`offset` | `playlists`、`total`、`more`、`cat` |
| `/top/playlist/highquality` | `cat`、`limit`、`before`（上一页最后一个歌单的 updateTime） | `playlists`、`total`、`more`、`lasttime` |
| `/playlist/catlist` | — | `categories`、`sub`、`all` |
| `/playlist/hot` | — | `tags` |
| `/style/list` | — | `data` |
| `/style/detail` | `tagId` | — |
| `/style/playlist` `/style/song` `/style/artist` | `tagId`、`cursor`、`size`（`/style/song` 还有 `sort`） | — |
| `/style/preference` | —（需登录） | `data`（游客为空） |
| `/dj/recommend` | — | `djRadios`、`name` |
| `/dj/program/toplist` | `limit`、`offset` | `toplist`、`updateTime` |
| `/dj/program/detail` | `id` | — |
| `/dj/detail` | `rid` | — |
| `/dj/program` | `rid`、`limit`、`offset`、`asc` | — |
| `/top/artists` | `limit`、`offset` | `artists`、`more` |
| `/toplist/artist` | `type` | `list` |
| `/voicelist/detail` | `id` | — |
| `/voicelist/list` | `voiceListId`、`limit`、`offset` | — |

以上读接口都会被后端缓存 2 分钟（按参数和登录身份区分）；需要强制刷新就带上 `timestamp=<毫秒时间戳>`。
