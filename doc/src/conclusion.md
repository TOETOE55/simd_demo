# 未来畅想

不过目前SIMD还是很难实现工程化，

1. 很难做到跨端——不同平台提供的SIMD指令都不同，差异很难抹平；甚至同平台中不同SIMD指令集之间差异都很大，本身就很难用一套通用的接口来描述。如果强行封装，要么就会丢掉一些功能，要么就缺少效率。这里就是一些trade off。
2. 很少语言有向量计算/并行计算的first-class支持，首先这便阻碍了设计向量计算、并行计算的抽象；另外缺乏相关语义，编译器读不懂便很难做优化，利用机器本身自带的一些向量计算、并行计算的能力。
3. 能利用向量化加速的算法还不多。



但我觉得事情在慢慢发生变化，硬件方面，由于AI的兴起，人们对算力的要求又上一个台阶，异构计算似乎慢慢走向主流；软件方面，现在一些新的语言也正开始把向量计算，并行计算考虑到语言设计中，比如说bend/mojo/zig等等。

于是最近便产生了这样一个想法：如果说这这代编程语言是由haskell, typescript, rust等语言带来的类型系统的革命——强大的类型系统能让我们描述管理各种抽象，且能保证类型安全。

那么，下一代编程语言我觉得应该是性能的革命。目前我认为不存在足够通用且好用的语言让我们轻松地在业务上利用起越来越多异构计算（gpu，tpu，fpga等）带来的优势。新一代的语言我觉得应该：

1. 除了能对标量计算进行编程外，同时还能方便地对向量、矩阵、张量进行编程，甚至还能对模拟计算进行编程（？
2. 能进行更精细的并发控制，能更好利用起并行计算的能力。
3. 既机器友好，也程序员友好。能写高等抽象，并能编译到各种异构的机器上。


不过说到底，还是编译器的革命，PL的革命，未来的编译器能把人和机器无缝连接起来。