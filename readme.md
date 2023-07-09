# JVM written in Rust
This project was not designed to be fast, efficient, reliable, or stable. It was just a fun
project. I wrote most of this code between my classes at university between June 2021 and
January 2022. This was not inspired by any university or professional projects, and I did not
collaborate with anyone else on this project (If I had, the quality probably would have been
better). Because I originally never intended for other people to see this, the code is a mess.
There are random test files in the repository, spaghetti code everywhere, heaps of miscellaneous
bad/commented/undocumented/incomplete code. View it at your own risk.

I initially started this project by wanting to write a simple program to read Java class files.
After doing that, I wondered if I could make it simulate a java program. This is what lead me down
the rabbit hole and resulted in this project. For this reason, I implemented this solely by reading
[Java SE 17 Virtual Machine Specification], comparing it with my local Java installations, and
occasionally trying to hunt through [github.com/openjdk/jdk] when I encountered something
unexpected. Much of my development was based on solving problems I encountered as I worked since I
was not following any guides or other reference information.

**What it can do:**
 - Find and parse Java 17 class and JAR files along the class path.
 - Run simple Java 8 programs. If I recall correctly, this was because Java 8 was the last version
   to include a `src.zip` containing all the classes within the standard library
 - Load shared libraries and call java native interface functions
 - Basic multithreading. I added multithreading because I needed it for my super simple
   "Hello World!" test. To call `System.out.println` it needed to get the default charset which was
   supposed to be thread local, and it needed a second thread to manage thread local instances. The
   real JVM cheats a bit by preloading some commonly used resources so this only happens if you
   do something special with the character sets. I just quickly hacked together something to get it
   to work, so I am not happy with the current thread safety and soundness implementation. This is
   one of my biggest regrets for the project since I botched one of the areas which Rust should
   have given an advantage. I may end up coming back to fix this later.
 - Run on both Linux and Windows. I initially started this project with the intention of it being
   linux only for simplicity. I use a Windows laptop and would test it with WSL. Eventually I
   decided that I wanted to be able to run it on Windows as well for ease of testing. This was a
   challenge, but mostly just involved fighting the linker to get it work play nice with other
   shared libraries in the JDK.

**What it can't do:**
 - Class loader reflection
 - Use some bytecode instructions
   - `invokedynamic`: This gets called in some cases when you use a lambda expression. It had some
      weird requirements I did not fully understand when I tried to use it with JDK 8 libraries. It
      also had requirements regarding how reflection worked which conflicted with what I had done
      previously. I could probably make it work, but since it didn't come up in the simple test
      programs I was using, I just ignored it.
   - `wide`: This instruction wraps a few other instructions and modifies them to allow for larger
      16-bit index fields. You would think this would be important, but it just never really came
      up.
 - Work with the components of a JDK newer that Java 8
 - JIT compilation. I started some work in this area, but I wouldn't say I really reached a
   satisfactory point for it. I used LLVM for a tiny handful of instructions, but it won't use
   any of it while running and never progressed further than a small experiment. I have since
   learned a lot more about LLVM and how it can be used in compilation, so I may eventually
   revisit this.
 - *Probably a bunch of other stuff I forgot because I haven't worked on this project in a couple
   of years*


[Java SE 17 Virtual Machine Specification]: https://docs.oracle.com/javase/specs/jvms/se17/html/index.html
[github.com/openjdk/jdk]: https://github.com/openjdk/jdk
