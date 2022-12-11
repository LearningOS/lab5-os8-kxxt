# 总结你实现的功能

添加了一个死锁检测器类 `DeadlockDetector`，它封装了使用银行家算法来检测死锁、追踪资源使用情况的代码，为了简单，我根据实验中的提示，分别给 Mutex 和 Semaphore 在 PCB 中分配了一个 DeadlockDetector。
在实现各个系统调用的函数里，我们不需要去显式的维护各种数据结构，只需要调用 `DeadlockDetector` 封装的方法即可，这遵循了软件工程的原理。

用时：大约 8 小时。（理解这个算法很费时间，理解了之后我还是不太明白它的原理。我一开始把算法理解错了，没注意算法第二步是对任意的 j ，这导致我调试了很久）

# 问答题

> 在我们的多线程实现中，当主线程 (即 0 号线程) 退出时，视为整个进程退出， 此时需要结束该进程管理的所有线程并回收其资源。 - 需要回收的资源有哪些？ - 其他线程的 TaskControlBlock 可能在哪些位置被引用，分别是否需要回收，为什么？

回收它打开的文件，它的 PID 和分配的互斥锁、信号量、条件变量，所有线程的 kernel stack、trap context 还有其他内存资源，它的子进程会被挂到 init 进程下。

> 对比以下两种 `Mutex.unlock` 的实现，二者有什么区别？这些区别可能会导致什么问题？
>
> ```rust
> impl Mutex for Mutex1 {
>     fn unlock(&self) {
>         let mut mutex_inner = self.inner.exclusive_access();
>         assert!(mutex_inner.locked);
>         mutex_inner.locked = false;
>         if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
>             add_task(waking_task);
>         }
>     }
> }
>
> impl Mutex for Mutex2 {
>     fn unlock(&self) {
>         let mut mutex_inner = self.inner.exclusive_access();
>         assert!(mutex_inner.locked);
>         if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
>             add_task(waking_task);
>         } else {
>             mutex_inner.locked = false;
>         }
>     }
> }
> ```

```

Mutex1 的 unlock 总是将 `.locked` 重置成 `false`. 这在不改变 `lock` 的实现时是有问题的。
被唤醒的线程认为自己取得锁，但是其他线程此时可以二次加锁，造成有两个线程同时在临界区的 bug 出现。
```
