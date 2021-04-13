#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>

pthread_mutex_t reader_lock;
pthread_mutex_t writer_lock;
pthread_mutex_t writer_wait_lock;
pthread_cond_t writer_wait_cond;

// this is protected by read_lock
int reader_n;

// these are protected by writer_wait_lock
int next_writer_id;
int last_writer_id;

void reader() {
  pthread_mutex_lock(&reader_lock);
  if (reader_n == 0) {
    pthread_mutex_lock(&writer_lock);
  } else {
    pthread_mutex_lock(&writer_wait_lock);
    int wait_writer_id = next_writer_id - 1;
    while (last_writer_id < wait_writer_id) {
      pthread_cond_wait(&writer_wait_cond, &writer_wait_lock);
    }
    pthread_mutex_unlock(&writer_wait_lock);
  }
  reader_n++;
  pthread_mutex_unlock(&reader_lock);

  // critical section start
  printf("reader!\n");
  // critical section end

  pthread_mutex_lock(&reader_lock);
  reader_n--;
  if (reader_n == 0) {
    pthread_mutex_unlock(&writer_lock);
  }
  pthread_mutex_unlock(&reader_lock);
}

void writer() {
  pthread_mutex_lock(&writer_wait_lock);
  int writer_id = next_writer_id++;
  pthread_mutex_unlock(&writer_wait_lock);
  pthread_mutex_lock(&writer_lock);
  pthread_mutex_lock(&writer_wait_lock);
  last_writer_id = writer_id;
  pthread_mutex_unlock(&writer_wait_lock);

  // critical section start
  printf("writer!\n");
  // critical section end

  pthread_cond_broadcast(&writer_wait_cond);
  pthread_mutex_unlock(&writer_lock);
}

int init() {
  reader_n = 0;
  last_writer_id = -1;
  next_writer_id = 0;
  pthread_mutex_init(&reader_lock, NULL);
  pthread_mutex_init(&writer_lock, NULL);
  pthread_mutex_init(&writer_wait_lock, NULL);
  pthread_cond_init(&writer_wait_cond, NULL);
}

void routine() {
  for (int i = 0; i < 1000; i++) {
    int rand_int = rand() % 4;
    if (rand_int == 0) {
      writer();
    } else {
      reader();
    }
  }
}

int main() {
  init();
  return 0;
}
