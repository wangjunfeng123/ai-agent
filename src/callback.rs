// callback
// 出现的时机
// 1.调用大模型前后
// 2.调用工具前后
// 2.1 工具前的调用：可以选择执不执行任务
// 2.2 工具后的调用：可以更改执行的结果\状态
// 3.整个任务跑完之后
pub mod approval;
pub mod context_optimizer;
pub mod search_compressor;
