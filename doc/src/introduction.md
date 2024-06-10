# SIMD简单介绍

SIMD是Single Instruction, Multiple Data的缩写，是现代CPU提供的一种同时（*单条指令*）对多个数据进行操作的指令。如果说以前的指令提供的是**标量计算**的能力，那么SIMD提供的就是**向量计算**的能力。

（还想补充一点的是，SIMD和超标量两种CPU能力是相互独立的，也就是说，SIMD的指令本身也可以像普通的标量计算的指令一样可以并行执行）



目前两个主流的架构都提供了SIMD指令：

* x86: XMM, SSE, AVX
* arm: neon, SVE
* wasm: simd128



我手头的电脑是11th Gen Intel(R) Core(TM) i7-1185G7 @ 3.00GHz（Tiger lake架构），所以后面便用x86中提供的SIMD指令来实现一些算法。

至于为什么要自己写SIMD代码、直接使用CPU提供的SIMD的intrinsic呢？主要有几个原因：

1. 目前大部分语言并没有对向量计算的抽象的能力，需要自己调用不同平台提供的SIMD函数（甚至自己写汇编）。

2. 编译器的自动向量化实在不够聪明（不过这也和语言本身没有类似特性有关），就算已经把代码写得十分“向量化”了，但生成的代码还是无法完全利用起SIMD的特性：

   ```c
   for (int i = 0; i < N; i+=8) {
       sum += A[i];
       sum += A[i+1];
       sum += A[i+2];
       sum += A[i+3];
       sum += A[i+4];
       sum += A[i+5];
       sum += A[i+6];
       sum += A[i+7];
   }
   ```

   理论上循环体里只要编译成`vaddps  (%rax), %ymm0, %ymm0`，就可以对八个加法一块完成，但gcc -O3愣是编译成了（并没有向量化）：

   ```assembly
           # vaddss是AVX里的标量加法，只把寄存器低位的值相加
           vaddss  (%rax), %xmm0, %xmm0
           vaddss  4(%rax), %xmm0, %xmm0
           vaddss  8(%rax), %xmm0, %xmm0
           vaddss  12(%rax), %xmm0, %xmm0
           vaddss  16(%rax), %xmm0, %xmm0
           vaddss  20(%rax), %xmm0, %xmm0
           vaddss  24(%rax), %xmm0, %xmm0
           vaddss  28(%rax), %xmm0, %xmm0
   ```

3. 各个平台的提供的SIMD指令都不一样（甚至单个平台内不同的SIMD指令集都不一样），几乎没有一套既通用、包含所有功能且有性能保证的可移植的SIMD接口使用。

