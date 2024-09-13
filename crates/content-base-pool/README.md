# content-base-pool

A task pool implementation for managing content-related tasks with priority and dependencies.

## Features

- Task queue with priority scheduling
- Support for CPU-bound and IO-bound tasks
- Task cancellation
- Task dependencies and subscriptions
- Notification system for task status updates

## Usage

```rust
use content_base_pool::{TaskPool, TaskPriority};
use content_base_context::ContentBaseCtx;
use content_base_task::ContentTaskType;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let content_base = ContentBaseCtx::new()?;
    let pool = TaskPool::new(&content_base, Some(4))?;

    pool.add_task(
        "file1",
        "path/to/file1",
        ContentTaskType::Parse,
        Some(TaskPriority::High),
        None,
    )
    .await?;

    // Add more tasks...

    Ok(())
}
```

## API

- `TaskPool::new()`: Create a new task pool
- `TaskPool::add_task()`: Add a new task to the pool
- `TaskPool::cancel_specific()`: Cancel a specific task
- `TaskPool::cancel_by_file()`: Cancel all tasks for a file
- `TaskPool::cancel_all()`: Cancel all tasks in the pool

### TaskPayload

`TaskPayload` 是一个枚举类型，用于表示可以发送到任务池的不同类型的消息或操作。它定义在 `payload.rs` 文件中，包含以下变体：

1. `Task(NewTaskPayload)`:
   - 这个变体用于添加新任务到任务池。
   - `NewTaskPayload` 包含了创建新任务所需的所有信息，如文件标识符、文件路径、任务类型、优先级和通知器。

2. `CancelByIdAndType(String, ContentTaskType)`:
   - 用于取消特定文件标识符和任务类型的任务。
   - 第一个 `String` 参数是文件标识符，`ContentTaskType` 是任务类型。

3. `CancelById(String)`:
   - 用于取消与特定文件标识符相关的所有任务。
   - `String` 参数是文件标识符。

4. `CancelAll`:
   - 用于取消任务池中的所有任务。

这个枚举的主要作用是为任务池提供一个统一的消息接口。当用户想要执行某个操作（如添加任务或取消任务）时，会创建相应的 `TaskPayload` 实例，并将其发送到任务池的消息通道。任务池的主循环会接收这些消息，并根据消息类型执行相应的操作。

### TaskId

`TaskId` 用于唯一标识任务池中的每个任务。

```rust
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct TaskId {
    file_identifier: String,
    task_type: ContentTaskType,
}
```
1. 主要组成部分：
   - `file_identifier`: 一个字符串，用于标识与任务相关的文件。
   - `task_type`: 一个 `ContentTaskType` 枚举，表示任务的类型。

2. 主要方法：
   - `new`: 创建新的 TaskId 实例。
   - `to_store_key`: 生成用于存储的键，格式为 "file_identifier:task_type"。
   - `file_identifier`: 获取文件标识符。
   - `task_type`: 获取任务类型。

3. 实现 `Display` trait：
   - 允许 TaskId 被格式化为字符串，格式为 "file_identifier:task_type"。

`TaskId` 的主要作用是：

1. 唯一标识：在任务池中唯一标识每个任务，便于任务的管理、查找和操作。

2. 任务映射：用作任务存储和映射的键，允许快速查找和访问特定任务。

3. 任务区分：通过组合文件标识符和任务类型，可以区分同一文件的不同类型任务，或不同文件的相同类型任务。

4. 灵活性：通过分离文件标识符和任务类型，提供了灵活的任务取消和管理机制，如按文件或按任务类型取消任务。

5. 序列化友好：实现了 `Display` trait，便于日志记录和调试。


## 任务池

任务池的主循环在 `TaskPool::new()` 方法中初始化。它会一直运行，直到程序结束或任务池被明确关闭（当前实现中没有提供关闭机制）。

详细过程：

1. 初始化时机：
   当调用 `TaskPool::new()` 创建一个新的任务池实例时，主循环就被初始化了。

2. 主循环的创建：
   在 `TaskPool::new()` 方法中，有这样一段代码：

   ```rust
   let (tx, mut rx) = mpsc::channel(512);

   // ...

   // loop for message
   tokio::spawn(async move {
       while let Some(payload) = rx.recv().await {
           // 处理接收到的消息
           // ...
       }
   });
   ```

3. 持续运行：
   - 这个循环被包装在 `tokio::spawn()` 中，创建了一个新的异步任务。
   - `while let Some(payload) = rx.recv().await` 会持续监听消息通道，只要通道没有关闭，它就会一直运行。
   - 每当有新的消息（任务或控制命令）到达时，循环就会处理这个消息。

4. 生命周期：
   - 这个循环会一直运行，直到以下情况之一发生：
     a. 程序终止。
     b. 所有的发送者（tx）都被丢弃，导致通道关闭。
     c. 显式地关闭任务池（当前实现中没有这个机制）。

5. 资源考虑：
   - 由于这是一个异步循环，它不会持续占用 CPU 资源。当没有消息时，它会有效地"睡眠"。
   - 只有在有消息需要处理时，它才会被唤醒并消耗资源。

6. 多个循环：
   注意，除了这个主消息处理循环，`TaskPool::new()` 还创建了两个任务执行循环（一个用于 CPU 绑定任务，一个用于 IO 绑定任务）：

   ```rust
   tokio::spawn(async move {
       cpu_task_ctx.run(&cb).await;
   });

   tokio::spawn(async move {
       io_task_ctx.run(&cb).await;
   });
   ```

   这些循环也会持续运行，负责实际执行队列中的任务。

## 任务处理流程

当调用 `add_task` 方法后，任务的执行流程如下：

1. 消息传递：
   - `add_task` 方法创建一个 `NewTaskPayload` 对象，并将其包装在 `TaskPayload::Task` 中。
   - 这个 payload 通过 `self.tx.send()` 发送到任务池的消息通道。

2. 消息处理：
   - 在任务池的主循环中（由 `tokio::spawn` 创建的异步任务），不断从 `rx` 接收消息。
   - 当收到 `TaskPayload::Task` 消息时，进行以下处理：

3. 任务创建和初始化：
   - 创建 `Task` 和 `TaskId` 对象。
   - 检查任务是否已存在，如果存在则忽略。
   - 创建 `TaskInQueue` 对象，包含任务信息、优先级、取消令牌等。

4. 任务存储：
   - 将任务信息存储在 `task_mapping` 中。
   - 将任务 ID 和优先级添加到对应的任务队列（CPU 或 IO）中。

5. 优先级处理：
   - 如果没有可用的信号量（即所有工作线程都忙），检查是否有优先级较低的任务可以被取消。
   - 如果找到，取消该任务并将其重新加入队列。

6. 任务执行准备：
   - 通知任务执行器有新任务可用（`task_ctx.notifier.notify_one()`）。

7. 任务执行：
   - 在任务执行循环中（`run` 方法），等待信号量并从任务队列中获取最高优先级的任务。
   - 检查任务依赖，如果有未完成的依赖，创建这些依赖任务并重新安排当前任务。
   - 如果没有依赖或依赖已完成，启动一个新的异步任务来执行实际的工作。

8. 任务执行过程：
   - 更新任务状态和发送通知（如果配置了通知器）。
   - 执行实际的任务逻辑（`current_task.task.task_type.run()`）。
   - 处理任务完成、错误或取消的情况。

9. 任务完成后处理：
   - 释放信号量。
   - 从各种映射和队列中移除任务信息。
   - 检查并更新依赖关系，可能会触发其他等待中的任务。

10. 循环继续：
    - 任务执行器继续循环，处理队列中的下一个任务。
