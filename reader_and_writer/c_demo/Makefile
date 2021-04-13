MY_IMPL = reader_first.c
# MY_IMPL = everyone_fair.c

tester: tester.c impl.o
	gcc -g -o $@ $^ -lpthread

impl.o: $(MY_IMPL)
	gcc -g -c -o impl.o $^ -lpthread

clean:
	rm -f impl.o tester tmp.txt