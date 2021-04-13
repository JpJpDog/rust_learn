#include <pthread.h>

pthread_mutex_t reader_lock;
pthread_mutex_t writer_lock;
int reader_n;

void reader() {
  pthread_mutex_lock(&reader_lock);
  if (reader_n++ == 0) {
    pthread_mutex_lock(&writer_lock);
  }
  pthread_mutex_unlock(&reader_lock);

  // critical section

  pthread_mutex_lock(&reader_lock);
  if (--reader_n == 0) {
    pthread_mutex_unlock(&writer_lock);
  }
  pthread_mutex_unlock(&reader_lock);
}

void writer() {
  pthread_mutex_lock(&writer_lock);

  // critical section

  pthread_mutex_unlock(&writer_lock);
}

int init() {
  reader_n = 0;
  pthread_mutex_init(&reader_lock, NULL);
  pthread_mutex_init(&writer_lock, NULL);
}

int main() {
  init();
  return 0;
}
