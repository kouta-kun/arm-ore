#include "syscall.hpp"
#include "console.hpp"

size_t SYSCALL(size_t syscall) {
  size_t result;
  asm("@ 0 parameter syscall\n"
      "mov r7, %[syscall]\n"
      "swi #0\n"
      "mov %[output], r0\n"
      : [output] "=r" (result)
      : [syscall] "r" (syscall)
      : "r7", "r0");
  return result;
}

size_t SYSCALL(size_t syscall, size_t arg1) {
  size_t result;

  asm("@ 1 parameter syscall\n"
      "mov r7, %[syscall]\n"
      "mov r1, %[arg1]\n"
      "swi #0\n"
      "mov %[output], r0\n"
      : [output] "=r" (result)
      : [syscall] "r" (syscall), [arg1] "r" (arg1)
      : "r7", "r0", "r1");
  return result;  
}

size_t SYSCALL(size_t syscall, size_t arg1, size_t arg2) {
  size_t result;

  asm("@ 2 parameter syscall\n"
      "mov r7, %[syscall]\n"
      "mov r1, %[arg1]\n"
      "mov r2, %[arg2]\n"
      "swi #0\n"
      "mov %[output], r0\n"
      : [output] "=r" (result)
      : [syscall] "r" (syscall), [arg1] "r" (arg1), [arg2] "r" (arg2)
      : "r7", "r0", "r1", "r2");
  return result;  
}


size_t SYSCALL(size_t syscall, size_t arg1, size_t arg2, size_t arg3, size_t arg4) {
  size_t result;
  asm("@ 4 parameter syscall\n"
      "mov r7, %[syscall]\n"
      "mov r1, %[arg1]\n"
      "mov r2, %[arg2]\n"
      "mov r3, %[arg3]\n"
      "mov r4, %[arg4]\n"
      "swi #0\n"
      "mov %[output], r0\n"
      : [output] "=r" (result)
      : [syscall] "r" (syscall), [arg1] "r" (arg1), [arg2] "r" (arg2), [arg3] "r" (arg3), [arg4] "r" (arg4)
      :  "r7", "r0", "r1", "r2", "r3", "r4");
  return result;  
}
