#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>

#include "reader_writer.h"

#define debug

static pthread_mutex_t reader_lock;
static pthread_mutex_t writer_lock;
static pthread_cond_t writer_wait_cond;

// this is protected by read_lock
static int reader_n;
static int next_writer_id;
static int last_writer_id;

void reader(void (*func)(void*), void* args) {
  pthread_mutex_lock(&reader_lock);
  if (reader_n == 0) {
    pthread_mutex_lock(&writer_lock);
  } else {
    int wait_writer_id = next_writer_id - 1;
    while (last_writer_id < wait_writer_id) {
      pthread_cond_wait(&writer_wait_cond, &reader_lock);
    }
  }
  reader_n++;
  pthread_mutex_unlock(&reader_lock);

#ifdef debug
  printf("### reader! read_n: %d\n", reader_n);
  fflush(stdout);
#endif
  func(args);

  pthread_mutex_lock(&reader_lock);
  if (--reader_n == 0) {
    pthread_mutex_unlock(&writer_lock);
  }
  pthread_mutex_unlock(&reader_lock);
}

void writer(void (*func)(void*), void* args) {
  pthread_mutex_lock(&reader_lock);
  int writer_id = next_writer_id++;
  pthread_mutex_unlock(&reader_lock);
  pthread_mutex_lock(&writer_lock);
  last_writer_id = writer_id;

#ifdef debug
  printf("### writer! cur writer id: %d\n", writer_id);
  fflush(stdout);
#endif
  func(args);

  pthread_cond_broadcast(&writer_wait_cond);
  pthread_mutex_unlock(&writer_lock);
}

void init_reader_writer() {
  reader_n = 0;
  last_writer_id = -1;
  next_writer_id = 0;
  pthread_mutex_init(&reader_lock, NULL);
  pthread_mutex_init(&writer_lock, NULL);
  pthread_cond_init(&writer_wait_cond, NULL);
}
