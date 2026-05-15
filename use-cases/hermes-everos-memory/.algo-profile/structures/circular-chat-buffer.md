---
algorithm: Circular Buffer
category: structures
complexity_time: O(1)
complexity_space: O(k)
used_in: raven-console/src/tui.rs
date: 2026-05-15
---

## Why This Was Chosen

Hermes Chat transcript 是固定窗口队列：新消息追加，旧消息淘汰。`VecDeque`
更贴合 FIFO/ring-buffer 语义，避免 `Vec` 从头部 `drain` 时搬移元素。

## Implementation Notes

`CHAT_HISTORY_LIMIT` 固定为 24。每次追加前检查容量，满了就 `pop_front()`，
再 `push_back()`，让长期运行 TUI 的 transcript 更新保持 O(1)。

## Reference

javascript-algorithms data structures decision guide: Circular Queue / Queue.
