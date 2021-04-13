## Introduction
1. this is a simple "reader writer lock" demo implemented by C
2. "reader_writer.h" is an interface of the lock
3. "read_first.c" is a simple implementation of "reader_writer.h". It is an example in most text book. It gives reader priority because reader can enter the critical section when a writer is waiting.
4. "everyone_fair.c" is a more complicated implementation. It loses some concurrency but add fairness. It refers to "ticket lock". A waiting writer will block the coming reader and give them an *id* about the writer. When the writer enter the critical section. Those waiting reader whose *id* is less than the just finished writer will rush into critical section. It can promise the writer will not starve.
5. I also provide a simple test program. It makes a statistic of the running order of "reader routine" and "writing routine". And calculate the avg at the end. It may partly prove that my implementation make sense.
## Usage
1. In the "Makefile", you can annotate "MY_IMPL". It will shift the implementation of the lock.
2. Type in "make" and then "./tester" or "./tester >tmp.txt" can run the test.
3. In "everyone_fair.c", there is an macro "debug". You can deannotate it to get debug information.