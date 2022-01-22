#ifndef __SYSCALL_HPP
#define __SYSCALL_HPP
#include <stddef.h>

size_t SYSCALL(size_t syscall);
size_t SYSCALL(size_t syscall, size_t arg1);
size_t SYSCALL(size_t syscall, size_t arg1, size_t arg2);
size_t SYSCALL(size_t syscall, size_t arg1, size_t arg2);
size_t SYSCALL(size_t syscall, size_t arg1, size_t arg2, size_t arg3, size_t arg4);
#endif
