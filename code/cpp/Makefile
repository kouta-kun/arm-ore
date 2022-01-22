main.iso: main.elf readme.txt
	mkisofs -o $@ $^

main.elf: main.o stdlib/syscall.o stdlib/console.o stdlib/memalloc.o stdlib/fileio.o stdlib/graphics.o
	arm-unknown-eabi-g++ $^ -o $@

%.o: %.cpp
	@echo COMPILING $<
	arm-unknown-eabi-g++ -fno-exceptions $< -o $@ -ffunction-sections -fdata-sections -nostartfiles -mcpu=arm946e-s -c


.PHONY: clean


clean:
	rm -f stdlib/*.o *.o main.elf main.iso *.S stdlib/*.S
