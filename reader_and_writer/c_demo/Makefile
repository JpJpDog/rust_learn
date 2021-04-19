# MY_IMPL = reader_first.c
MY_IMPL = everyone_fair.c

tester: tester.c impl.o
	gcc -O3 -o $@ $^ -lpthread

impl.o: $(MY_IMPL)
	gcc -O3 -c -o impl.o $^ -lpthread

clean:
	rm -f impl.o tester tmp.txt