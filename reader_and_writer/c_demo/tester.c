#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <unistd.h>

#include "reader_writer.h"

int test_data = 0;

int writer_n = 0;
int reader_n = 0;
int order = 0;
int writer_order = 0;
int reader_order = 0;

const int kMaxSleep = 20000;
const int kMinSleep = 10000;

const int kThreadN = 20;
const int kWriterThreadN = 5;
const int kReaderLoopN = 200;
const int kWriterLoopN = 40;

int make_rand() { return rand() % (kMaxSleep - kMinSleep) + kMinSleep; }

void reader_routine(void* args) {
  printf("data: %d\n", test_data);
  reader_n++;
  reader_order += order++;
  usleep(make_rand());
}

void writer_routine(void* args) {
  printf("data: %d\n", ++test_data);
  writer_n++;
  writer_order += order++;
  usleep(make_rand());
}

void* routine(void* args) {
  int arg = *(int*)args;
  free(args);
  if (arg == 0) {
    for (int i = 0; i < kReaderLoopN; i++) {
      reader(reader_routine, NULL);
    }
  } else {
    for (int i = 0; i < kWriterLoopN; i++) {
      writer(writer_routine, NULL);
    }
  }
}

void shuffle(int* arr, int len) {
  if (len == 1) return;
  for (int i = len - 1; i >= 0; i--) {
    size_t j = i + rand() / (RAND_MAX / (len - i) + 1);
    int tmp = arr[j];
    arr[j] = arr[i];
    arr[i] = tmp;
  }
}

int main() {
  srand(time(NULL));
  init_reader_writer();
  pthread_t t_ids[kThreadN];
  int is_writer[kThreadN];
  for (int i = 0; i < kWriterThreadN; i++) {
    is_writer[i] = 1;
  }
  for (int i = kWriterThreadN; i < kThreadN; i++) {
    is_writer[i] = 0;
  }
  shuffle(is_writer, kThreadN);
  for (int i = 0; i < kThreadN; i++) {
    int* arg = (int*)malloc(sizeof(int));
    *arg = is_writer[i];
    pthread_create(t_ids + i, NULL, routine, arg);
  }
  for (int i = 0; i < kThreadN; i++) {
    pthread_join(t_ids[i], NULL);
  }
  if (reader_n == 0 || writer_order == 0) {
    printf("error! try again!\n");
    return 1;
  }
  printf("reader n: %d, avg order: %lf\n", reader_n,
         ((double)reader_order) / reader_n);
  printf("writer n: %d, avg order: %lf\n", writer_n,
         ((double)writer_order) / writer_n);
  return 0;
}