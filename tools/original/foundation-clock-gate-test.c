#define _GNU_SOURCE
#include <errno.h>
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/syscall.h>
#include <sys/time.h>
#include <time.h>
#include <unistd.h>
void glXSwapBuffers(void *,unsigned long);
int XNextEvent(void *,void *);
int XCheckTypedEvent(void *,int,void *);
int XCheckIfEvent(void *,void *,int (*)(void *,void *,char *),char *);
static unsigned assertions;
#define CHECK(x) do { assertions++; if(!(x)){fprintf(stderr,"failed assertion %u: %s\n",assertions,#x);exit(1);} }while(0)
static uint64_t ns(struct timespec t){return (uint64_t)t.tv_sec*1000000000+t.tv_nsec;}
static void *worker(void *unused) {
    (void)unused;struct timespec before,host,after;
    CHECK(clock_gettime(CLOCK_MONOTONIC,&before)==0);
    CHECK(syscall(SYS_clock_gettime,CLOCK_MONOTONIC,&host)==0);
    CHECK(ns(before)>ns(host)-100000000 && ns(before)<ns(host)+100000000);
    struct timespec wait={0,2000000};CHECK(nanosleep(&wait,NULL)==0);
    CHECK(clock_gettime(CLOCK_MONOTONIC,&after)==0);CHECK(ns(after)>=ns(before)+1000000);
    return NULL;
}
static int predicate(void *display,void *event,char *argument) {
    CHECK(display==(void*)123);CHECK(*(int*)event==2);CHECK(argument==(char*)456);return 1;
}
int main(void) {
    struct timespec now,cpu,real;struct timeval wall;time_t seconds=0;
    CHECK(clock_gettime(CLOCK_MONOTONIC,&now)==0);CHECK(ns(now)==UINT64_C(1000000000000));
    CHECK(clock_gettime(CLOCK_REALTIME,&now)==0);CHECK(ns(now)==UINT64_C(1700000000000000000));
    CHECK(gettimeofday(&wall,NULL)==0);CHECK(wall.tv_sec==1700000000 && wall.tv_usec==0);
    CHECK(time(&seconds)==1700000000 && seconds==1700000000);
    CHECK(clock_gettime(CLOCK_THREAD_CPUTIME_ID,&cpu)==0);CHECK(ns(cpu)<UINT64_C(1000000000000));
    errno=0;CHECK(clock_gettime((clockid_t)-321,&now)==-1);CHECK(errno==EINVAL);
    struct timespec invalid={0,1000000000};errno=0;CHECK(nanosleep(&invalid,NULL)==-1);CHECK(errno==EINVAL);
    struct timespec duration={0,5000000},remaining={123,456};
    CHECK(nanosleep(&duration,&remaining)==0);CHECK(remaining.tv_sec==0 && remaining.tv_nsec==0);
    CHECK(usleep(5000)==0);CHECK(clock_gettime(CLOCK_MONOTONIC,&now)==0);CHECK(ns(now)==UINT64_C(1000000000000));
    pthread_t child;CHECK(pthread_create(&child,NULL,worker,NULL)==0);CHECK(pthread_join(child,NULL)==0);
    long event[24]={0};
    CHECK(XNextEvent((void*)123,event)==0);CHECK(*(int*)event==2);
    CHECK(XCheckTypedEvent((void*)123,3,event)==1);CHECK(*(int*)event==3);
    CHECK(XCheckIfEvent((void*)123,event,predicate,(char*)456)==1);
    CHECK(XCheckTypedEvent((void*)123,9,event)==1);CHECK(*(int*)event==9);
    CHECK(XCheckTypedEvent((void*)123,10,event)==1);CHECK(*(int*)event==10);
    glXSwapBuffers(NULL,0);
    CHECK(clock_gettime(CLOCK_MONOTONIC,&now)==0);CHECK(ns(now)==UINT64_C(1000016667000));
    CHECK(gettimeofday(&wall,NULL)==0);CHECK(wall.tv_sec==1700000000 && wall.tv_usec==16667);
    glXSwapBuffers(NULL,0);
    CHECK(clock_gettime(CLOCK_MONOTONIC,&now)==0);CHECK(ns(now)==UINT64_C(1000033334000));
    glXSwapBuffers(NULL,0);
    CHECK(clock_gettime(CLOCK_MONOTONIC,&now)==0);CHECK(ns(now)==UINT64_C(1000050001000));
    CHECK(syscall(SYS_clock_gettime,CLOCK_MONOTONIC,&real)==0);CHECK(ns(real)!=ns(now));
    printf("{\"status\":\"pass\",\"assertions\":%u}\n",assertions);fflush(stdout);
    glXSwapBuffers(NULL,0);
    return 0;
}
