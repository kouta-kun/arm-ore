main.iso: main.elf readme.txt
	mkisofs -o $@ $^

main.elf: main.o stdlib/syscall.o stdlib/console.o stdlib/memalloc.o stdlib/armmath.o stdlib/fileio.o
	ld.lld $^ -o $@

%.o: %.cpp
	clang++ -nostdlib -fno-exceptions -ffreestanding $< --target=armv4-unknown-none -o $@ -ffunction-sections -fdata-sections -nostartfiles -mcpu=arm946e-s -c


.PHONY: clean


clean:
	rm -f stdlib/*.o *.o main.elf main.iso
