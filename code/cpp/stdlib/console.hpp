#ifndef __CONSOLE_HPP
#define __CONSOLE_HPP
#include <stddef.h>
#include <stdint.h>
void printchar(char c);
void print(char* str);
void println(char* str);
void printint(int i);
char semibyte_to_hex(uint8_t i);

template<typename T>
void print_as_hex(T in) {
  print("0x");
  uint8_t *ptr = reinterpret_cast<uint8_t*>(&in);
  for(size_t i = sizeof(T); i > 0; i--) {
    uint8_t v = ptr[i-1];
    uint8_t a = uint8_t(v & 0x0F);
    uint8_t b = uint8_t((v & 0xF0) >> 4);
    printchar(semibyte_to_hex(b));
    printchar(semibyte_to_hex(a));
  }
  printchar('\n');
}
#endif
