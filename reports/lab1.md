
# 荣誉准则
在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

无

此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

https://learningos.cn/rCore-Tutorial-Guide-2024S/index.html

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。

<br>

# 简要描述实现功能

在 os/src/task/mod 文件下的 TaskManagerInner 结构体 中加入了 task_infos 数组，数组每个元素对应的是每个 task 的 TaskInfo,在 run_first_task 和 run_next_task 方法中分别对 task_infos数组进行处理；新加了两个方法 record_current_syscall 和 get_time_val ，分别记录当前系统调用的类型及次数、记录当前系统调用距离 task 第一次被调用的 时间差；<br>
在 os/src/syscall/mod 文件下 对每个系统调用进行处理时候，也对每个 task 的 task_infos 数组进行对应处理 

<br>

# 简答作业

<br>

## 1
 ```PageFault in application, bad addr = 0x0, bad instruction = 0x804003ac, kernel killed it. ```<br>
```IllegalInstruction in application, kernel killed it.```

## 2
1. 刚进入 __restore 时候， a0 代表的是内核栈的栈顶指针 sp，__restore 的两个使用场景是 __alltraps 执行完后 自然执行到 __restroe ，第二个是 __switch 执行后跳转到 __restore;
2. sstatus 保存的是进入用户态前的特权级信息<br>
   spec    保存的是进入内核态之前的最后一条指令的位置，有利于找到进入用户态后正确的执行位置<br>
   sscratch 保存的是进入用户态前内核栈的栈指针 sp; <br>
3. 跳过 x2 是因为 要给保存其他通用的寄存器腾出空间，而且用户/内核栈指针保存在 sscratch 中，必须通过 csrr 指令读到通用寄存器后才能使用
   跳过 x4 是因为 app 不需要用到
4. sp 指向的是用户栈，sscratch 指向的是内核栈
5. sret 指令，因为 sp 已近指向用户栈了
6. sp 指向的是内核栈， sscratch 指向的是用户栈
7. csrrw sp, sscratch, sp 指令？ 
