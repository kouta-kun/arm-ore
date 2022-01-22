#include "console.hpp"

char semibyte_to_hex(uint8_t i) {
  if(i < 10) {
    return i +'0';
  } else return (i-10) + 'A';
}

void printchar(char c) {
  static char *const output = reinterpret_cast<char*const>(0xFF000L);
  *output = c;
}


void print(char* str) {
  while(*str) printchar(*(str++));
}

void println(char* str) {
  print(str);
  printchar('\n');
}


int reverse_int(int i) {
  int x = 0;
  while(i > 0) {
    x *= 10;
    x += (i % 10);
    i /= 10;
  }
  return x;
}

void printint(int i) {
  if (i == 0) {
    printchar('0');
    return;
  }
  i = reverse_int(i);
  while(i > 0) {
    char c = (i%10)+'0';
    i /= 10;
    printchar(c);
  }
};
