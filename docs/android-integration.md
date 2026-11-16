# Android integration (replacing nodejs-mobile)

[中文在下方 / Chinese below](#中文)

The backend ships as `libneteasy_backend.so` (one per ABI). It exposes one JNI function and serves the same
HTTP API on `127.0.0.1:19800`, so `ApiClient` and the OkHttp cookie jar keep working unchanged.

## 1. Get the libraries

- CI: open the latest run of **CI** → artifact `jniLibs` (contains `arm64-v8a/`, `armeabi-v7a/`, `x86_64/`), or
- Release: tag `vX.Y.Z` → `neteasy-backend-jniLibs.zip` is attached to the GitHub release, or
- Local: `cargo install cargo-ndk && cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 -o jniLibs build --release --lib`

Copy the ABI folders into `app/src/main/jniLibs/`.

## 2. Kotlin bridge

The class name is fixed by the JNI symbol (`Java_top_yunov_neteasy_RustBackend_start`):

```kotlin
package top.yunov.neteasy

object RustBackend {
    init { System.loadLibrary("neteasy_backend") }

    /** Returns true once the port is bound and connectable; false if it was already running or the bind failed. */
    @JvmStatic external fun start(dataDir: String, port: Int): Boolean
}
```

## 3. Start it

Replace the asset-copy + `NodeJS.start(...)` part of `NodeService`:

```kotlin
override fun onCreate() {
    super.onCreate()
    startAsForeground()
    // device id / xeapi key / guest token are cached here, so warm starts are instant
    RustBackend.start(File(filesDir, "neteasy-backend").absolutePath, 19800)
}
```

No more `assets/nodejs-project`, no APK-update copy, no `libnode`, no `native-lib.cpp` / `CMakeLists.txt`,
no pnpm/patch Gradle tasks. `start` only returns after the listening socket is bound (at most a few ms), so the
connection-refused retry loop in `ApiClient` is no longer needed on cold start (it is harmless if kept).

## 4. Notes

- Logs go to logcat with tag `neteasy-backend`.
- Posting comments (`/comment/add`) is out of scope and will not be implemented; fetching comments works. Everything else the app calls is ported.
- Native method names are kept by the default R8 rules (`-keepclasseswithmembernames` for natives).

---

## 中文

后端以 `libneteasy_backend.so`（每个 ABI 一份）交付，只导出一个 JNI 函数，对外仍是 `127.0.0.1:19800` 的同一套
HTTP 接口，所以 `ApiClient` 和 OkHttp 的 CookieJar 都不用改。

### 1. 拿到动态库

- CI：打开最近一次 **CI** 运行，下载 artifact `jniLibs`（含 `arm64-v8a/`、`armeabi-v7a/`、`x86_64/`）；或
- Release：打 tag `vX.Y.Z`，GitHub release 会附带 `neteasy-backend-jniLibs.zip`；或
- 本地：`cargo install cargo-ndk && cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 -o jniLibs build --release --lib`

把各 ABI 目录拷到 `app/src/main/jniLibs/`。

### 2. Kotlin 桥接

类名由 JNI 符号决定（`Java_top_yunov_neteasy_RustBackend_start`），必须是 `top.yunov.neteasy.RustBackend`，
代码见上方英文部分。

### 3. 启动

把 `NodeService` 里“复制 assets + `NodeJS.start(...)`”换成：

```kotlin
RustBackend.start(File(filesDir, "neteasy-backend").absolutePath, 19800)
```

可以删掉：`assets/nodejs-project`、APK 更新时的复制逻辑、`libnode`、`native-lib.cpp` / `CMakeLists.txt`、
pnpm 安装和补丁相关的 Gradle 任务。端口在第一次网络请求之前就已绑定，冷启动时 `ApiClient` 里“连接被拒绝就重试”的
逻辑不再是必需的（保留也无害）。

### 4. 说明

- 日志在 logcat，tag 为 `neteasy-backend`。
- 发评论（`/comment/add`）不在范围内，不会实现；获取评论可用。App 用到的其余接口都已移植。
- 默认的 R8 规则会保留 native 方法名，无需额外 keep。
