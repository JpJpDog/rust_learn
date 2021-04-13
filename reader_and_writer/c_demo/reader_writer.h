#ifndef READER_WRITER_H
#define READER_WRITER_H

void init_reader_writer();

void reader(void (*func)(void*), void* args);

void writer(void (*func)(void*), void* args);

#endif