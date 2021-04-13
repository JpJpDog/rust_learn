#include <pthread.h>
#include <stdio.h>

#include "reader_writer.h"

static pthread_mutex_t reader_lock;
static pthread_mutex_t writer_lock;
static int reader_n;

void reader(void (*func)(void*), void* args) {
  pthread_mutex_lock(&reader_lock);
  if (reader_n++ == 0) {
    pthread_mutex_lock(&writer_lock);
  }
  pthread_mutex_unlock(&reader_lock);

  // printf("reader! read_n: %d!\n", reader_n);
  func(args);

  pthread_mutex_lock(&reader_lock);
  if (--reader_n == 0) {
    pthread_mutex_unlock(&writer_lock);
  }
  pthread_mutex_unlock(&reader_lock);
}

void writer(void (*func)(void*), void* args) {
  pthread_mutex_lock(&writer_lock);

  // printf("writer!\n");
  func(args);

  pthread_mutex_unlock(&writer_lock);
}

void init_reader_writer() {
  reader_n = 0;
  pthread_mutex_init(&reader_lock, NULL);
  pthread_mutex_init(&writer_lock, NULL);
}
