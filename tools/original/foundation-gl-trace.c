/* Graphics-only observer for documented OpenGL/X11 ABI calls.
 * All calls are forwarded unchanged. Reads cover only submitted vertex/index
 * arrays or documented GL buffer contents, never gameplay state/instructions.
 * A request file triggers one complete presented render frame, not a game tick.
 */
#define _GNU_SOURCE
#include <dlfcn.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <signal.h>
#include <time.h>
#include <math.h>
#include <stdatomic.h>

typedef unsigned int GLenum;
typedef unsigned int GLuint;
typedef int GLint;
typedef int GLsizei;
typedef unsigned char GLubyte;
typedef float GLfloat;
typedef double GLdouble;
typedef long GLsizeiptr;
typedef long GLintptr;
#define GL_FLOAT 0x1406
#define GL_DOUBLE 0x140A
#define GL_UNSIGNED_BYTE 0x1401
#define GL_UNSIGNED_SHORT 0x1403
#define GL_UNSIGNED_INT 0x1405
#define GL_ARRAY_BUFFER 0x8892
#define GL_ARRAY_BUFFER_BINDING 0x8894
#define GL_ELEMENT_ARRAY_BUFFER 0x8893
#define GL_ELEMENT_ARRAY_BUFFER_BINDING 0x8895
#define GL_RGBA 0x1908
#define GL_BGRA 0x80E1

struct Array { GLint size; GLenum type; GLsizei stride; const void *pointer; GLuint buffer; };
static __thread struct Array vertex_array, uv_array, color_array;
static __thread GLuint texture;
static __thread GLuint program;
static __thread int active;
static __thread int snapshot_requested;
static __thread unsigned frame_number;
static __thread unsigned draw_number;
static FILE *stream;
static const char *directory;
static const char *control;
static char label[128];
static int texture_width[65536], texture_height[65536];
static unsigned texture_epoch[65536];
static unsigned cache_snapshot_epoch[65536];
void foundation_graphics_state(FILE *stream,unsigned texture);
static unsigned char used_texture[65536];
struct UniformName{GLuint program;GLint location;char name[128];};
static struct UniformName uniform_names[256];static unsigned uniform_name_count;

static void init(void) {
    if (stream) return;
    directory = getenv("ISAAC_GL_TRACE_DIR");
    control = getenv("ISAAC_GL_TRACE_CONTROL");
    if (!directory || !control) return;
    char path[2048]; snprintf(path, sizeof(path), "%s/commands.jsonl", directory);
    stream = fopen(path, "a");
    if (stream) setvbuf(stream, NULL, _IOLBF, 0);
}
static GLuint binding(GLenum which) {
    void (*real)(GLenum, GLint*) = dlsym(RTLD_NEXT, "glGetIntegerv");
    GLint value = 0; if(real) real(which, &value); return value;
}
static int enabled(GLenum which) {
    GLubyte (*real)(GLenum) = dlsym(RTLD_NEXT, "glIsEnabled");
    return real && real(which);
}
/* JSON has no NaN/Infinity numeric tokens. Preserve observed nonfinite values
 * as null, rather than silently dropping an otherwise diagnostic render frame. */
static void number(double value) {
    if(isfinite(value))fprintf(stream,"%.9g",value);else fputs("null",stream);
}
static int finite_components(const double *values,int size) {
    for(int i=0;i<size;i++)if(!isfinite(values[i]))return 0;
    return 1;
}
static void numbers(const double *values,int size) {
    fputc('[',stream);
    for(int i=0;i<size;i++){if(i)fputc(',',stream);number(values[i]);}
    fputc(']',stream);
}
static size_t scalar_size(GLenum type) {
    if (type == GL_FLOAT || type == GL_UNSIGNED_INT) return 4;
    if (type == GL_DOUBLE) return 8;
    if (type == GL_UNSIGNED_SHORT) return 2;
    if (type == GL_UNSIGNED_BYTE) return 1;
    return 0;
}
static int read_bytes(GLenum target, GLuint buffer, const void *pointer, size_t offset, size_t size, void *data) {
    if (buffer) {
        void (*real)(GLenum, GLintptr, GLsizeiptr, void*) = dlsym(RTLD_NEXT, "glGetBufferSubData");
        if (!real) return 0;
        real(target, (GLintptr)(uintptr_t)pointer + (GLintptr)offset, (GLsizeiptr)size, data);
        return 1;
    }
    if ((uintptr_t)pointer < 4096) return 0;
    memcpy(data, (const unsigned char*)pointer + offset, size); return 1;
}
static int components(struct Array array, unsigned index, double values[4]) {
    size_t width = scalar_size(array.type);
    if (!width || array.size < 1 || array.size > 4) return 0;
    unsigned char bytes[32]; size_t stride = array.stride ? array.stride : width*array.size;
    if(!read_bytes(GL_ARRAY_BUFFER, array.buffer, array.pointer, (size_t)index*stride, width*array.size, bytes)) return 0;
    for(int c=0;c<array.size;c++) {
        if(array.type == GL_FLOAT) { float v; memcpy(&v,bytes+c*4,4); values[c]=v; }
        else if(array.type == GL_DOUBLE) { double v; memcpy(&v,bytes+c*8,8); values[c]=v; }
        else if(array.type == GL_UNSIGNED_BYTE) values[c]=bytes[c];
        else if(array.type == GL_UNSIGNED_SHORT) { uint16_t v; memcpy(&v,bytes+c*2,2); values[c]=v; }
        else { uint32_t v; memcpy(&v,bytes+c*4,4); values[c]=v; }
    }
    return 1;
}
void glBindTexture(GLenum target, GLuint id) {
    void (*real)(GLenum,GLuint)=dlsym(RTLD_NEXT,"glBindTexture"); real(target,id); texture=id;
}
void glTexImage2D(GLenum target,GLint level,GLint internal,GLsizei width,GLsizei height,GLint border,GLenum format,GLenum type,const void *pixels) {
    void (*real)(GLenum,GLint,GLint,GLsizei,GLsizei,GLint,GLenum,GLenum,const void*)=dlsym(RTLD_NEXT,"glTexImage2D");
    real(target,level,internal,width,height,border,format,type,pixels); init();
    if(level==0 && texture<65536){texture_width[texture]=width;texture_height[texture]=height;texture_epoch[texture]++;}
    if (!stream) return;
    fprintf(stream,"{\"op\":\"texture\",\"id\":%u,\"width\":%d,\"height\":%d,\"format\":%u,\"type\":%u,\"level\":%d",texture,width,height,format,type,level);
    if (level==0 && (uintptr_t)pixels>=4096 && binding(0x88ef)==0 && width>=400 && height>=400 && width<=2048 && height<=2048 && pixels && type==GL_UNSIGNED_BYTE && (format==GL_RGBA || format==GL_BGRA)) {
        char path[2048]; snprintf(path,sizeof(path),"%s/texture-%u-%dx%d.rgba",directory,texture,width,height);
        FILE *image=fopen(path,"wb");
        if(image) { fwrite(pixels,4,(size_t)width*height,image); fclose(image); fprintf(stream,",\"rawPath\":\"%s\"",path); }
    }
    fprintf(stream,"}\n");
}
void glTexSubImage2D(GLenum target,GLint level,GLint xoffset,GLint yoffset,GLsizei width,GLsizei height,GLenum format,GLenum type,const void *pixels) {
    void (*real)(GLenum,GLint,GLint,GLint,GLsizei,GLsizei,GLenum,GLenum,const void*)=dlsym(RTLD_NEXT,"glTexSubImage2D");
    real(target,level,xoffset,yoffset,width,height,format,type,pixels);init();
    if(!stream)return;
    fprintf(stream,"{\"op\":\"textureUpdate\",\"id\":%u,\"x\":%d,\"y\":%d,\"width\":%d,\"height\":%d,\"format\":%u,\"type\":%u}\n",texture,xoffset,yoffset,width,height,format,type);
    if(level==0 && texture<65536 && (uintptr_t)pixels>=4096 && binding(0x88ef)==0 && type==GL_UNSIGNED_BYTE && (format==GL_RGBA||format==GL_BGRA)) {
        int tw=texture_width[texture],th=texture_height[texture];
        if(tw>=400 && th>=400 && tw<=2048 && th<=2048 && xoffset>=0&&yoffset>=0&&xoffset+width<=tw&&yoffset+height<=th){
            char path[2048];snprintf(path,sizeof(path),"%s/texture-%u-%dx%d.rgba",directory,texture,tw,th);
            FILE *image=fopen(path,"r+b");if(!image)image=fopen(path,"w+b");
            GLint rowlength=0;void(*get)(GLenum,GLint*)=dlsym(RTLD_NEXT,"glGetIntegerv");get(0x0CF2,&rowlength);size_t stride=(size_t)(rowlength?rowlength:width)*4;
            if(image){for(int y=0;y<height;y++){fseek(image,((long)(yoffset+y)*tw+xoffset)*4,SEEK_SET);fwrite((const unsigned char*)pixels+y*stride,4,width,image);}fclose(image);}
        }
    }
}

void glVertexPointer(GLint size,GLenum type,GLsizei stride,const void *pointer) {
    void (*real)(GLint,GLenum,GLsizei,const void*)=dlsym(RTLD_NEXT,"glVertexPointer"); real(size,type,stride,pointer);
    vertex_array=(struct Array){size,type,stride,pointer,binding(GL_ARRAY_BUFFER_BINDING)};
}
void glTexCoordPointer(GLint size,GLenum type,GLsizei stride,const void *pointer) {
    void (*real)(GLint,GLenum,GLsizei,const void*)=dlsym(RTLD_NEXT,"glTexCoordPointer"); real(size,type,stride,pointer);
    uv_array=(struct Array){size,type,stride,pointer,binding(GL_ARRAY_BUFFER_BINDING)};
}
void glColorPointer(GLint size,GLenum type,GLsizei stride,const void *pointer) {
    void (*real)(GLint,GLenum,GLsizei,const void*)=dlsym(RTLD_NEXT,"glColorPointer"); real(size,type,stride,pointer);
    color_array=(struct Array){size,type,stride,pointer,binding(GL_ARRAY_BUFFER_BINDING)};
}
static void snapshot_cache_source(GLuint id){
    if(!id||id>=65536||texture_width[id]<400||texture_height[id]<400||cache_snapshot_epoch[id]==texture_epoch[id])return;
    cache_snapshot_epoch[id]=texture_epoch[id];
    if(binding(0x88ed)||binding(0x0D02)||binding(0x0D03)||binding(0x0D04))return;
    int w=texture_width[id],h=texture_height[id];if(w>2048||h>2048)return;
    void(*bind)(GLenum,GLuint)=dlsym(RTLD_NEXT,"glBindTexture");void(*get)(GLenum,GLint,GLenum,GLenum,void*)=dlsym(RTLD_NEXT,"glGetTexImage");if(!get)return;
    GLuint previous=binding(0x8069);unsigned char *data=malloc((size_t)w*h*4);if(!data)return;
    bind(0x0DE1,id);get(0x0DE1,0,GL_RGBA,GL_UNSIGNED_BYTE,data);bind(0x0DE1,previous);
    char path[2048];snprintf(path,sizeof(path),"%s/texture-cache-epoch-%u-%u-%dx%d.rgba",directory,texture_epoch[id],id,w,h);
    FILE *image=fopen(path,"wb");if(image){fwrite(data,4,(size_t)w*h,image);fclose(image);fprintf(stream,"{\"op\":\"textureSnapshot\",\"stage\":\"cache-source\",\"id\":%u,\"epoch\":%u,\"frame\":%u,\"width\":%d,\"height\":%d,\"rawPath\":\"%s\",\"scope\":\"public GL cache source texture pixels\"}\n",id,texture_epoch[id],frame_number,w,h,path);}free(data);
}
static void snapshot_draw_pixels(const char *stage,unsigned serial){
    const char *target=getenv("ISAAC_GL_TRACE_PIXEL_DRAW_TEXTURE");
    if(!active||!snapshot_requested||!stream||!target||strtoul(target,NULL,10)!=texture)return;
    /* Respect the current read surface and pixel packing. These diagnostics do
     * not change any GL state or inspect shader/gameplay instructions. */
    if(binding(0x88ed)||binding(0x0D02)||binding(0x0D03)||binding(0x0D04))return;
    void(*get)(GLenum,GLint*)=dlsym(RTLD_NEXT,"glGetIntegerv");
    void(*read)(GLint,GLint,GLsizei,GLsizei,GLenum,GLenum,void*)=dlsym(RTLD_NEXT,"glReadPixels");
    if(!get||!read)return;
    GLint viewport[4]={0};get(0x0BA2,viewport);
    int w=viewport[2],h=viewport[3];if(w<1||h<1||w>3840||h>2160)return;
    GLint pack=binding(0x0D05);size_t row=(size_t)w*4;
    if(pack>0&&row%(unsigned)pack)return;
    unsigned char *data=malloc(row*h);if(!data)return;
    read(viewport[0],viewport[1],w,h,GL_RGBA,GL_UNSIGNED_BYTE,data);
    char path[2048];snprintf(path,sizeof(path),"%s/pixels-frame-%u-draw-%u-%s.rgba",directory,frame_number,serial,stage);
    FILE *file=fopen(path,"wb");
    if(file){fwrite(data,row,h,file);fclose(file);fprintf(stream,"{\"op\":\"drawPixels\",\"stage\":\"%s\",\"frame\":%u,\"draw\":%u,\"texture\":%u,\"viewport\":[%d,%d,%d,%d],\"readFramebuffer\":%u,\"readBuffer\":%u,\"origin\":\"bottom-left\",\"format\":\"RGBA8\",\"rawPath\":\"%s\",\"scope\":\"documented GL framebuffer pixels; no state mutation\"}\n",stage,frame_number,serial,texture,viewport[0],viewport[1],w,h,binding(0x8CAA),binding(0x0C02),path);}
    free(data);
}

void glDrawElements(GLenum mode,GLsizei count,GLenum type,const void *indices) {
    void (*real)(GLenum,GLsizei,GLenum,const void*)=dlsym(RTLD_NEXT,"glDrawElements");
    unsigned serial=active?++draw_number:0;
    if(active && texture<65536)used_texture[texture]=1;
    snapshot_draw_pixels("before",serial);
    GLuint framebuffer=binding(0x8CA6), attachment=0;
    if(framebuffer){
        void(*get)(GLenum,GLenum,GLenum,GLint*)=dlsym(RTLD_NEXT,"glGetFramebufferAttachmentParameteriv");
        if(!get)get=dlsym(RTLD_NEXT,"glGetFramebufferAttachmentParameterivEXT");
        if(get){GLint object=0;get(0x8D40,0x8CE0,0x8CD1,&object);attachment=object;}
    }
    int cache_draw=attachment<65536 && attachment && texture_width[attachment]>=400 && texture_width[attachment]<=1100 && texture_height[attachment]>=400 && texture_height[attachment]<=600;
    if(cache_draw && stream)snapshot_cache_source(texture);
    if((active || cache_draw) && stream && count>0 && count<=24576) {
        size_t width=scalar_size(type); unsigned char *data=width?malloc((size_t)count*width):NULL;
        GLuint element=binding(GL_ELEMENT_ARRAY_BUFFER_BINDING);
        if(data && read_bytes(GL_ELEMENT_ARRAY_BUFFER,element,indices,0,(size_t)count*width,data)) {
            unsigned max=0; for(int i=0;i<count;i++) { unsigned n=0; if(width==1)n=data[i];else if(width==2){uint16_t v;memcpy(&v,data+i*2,2);n=v;}else{uint32_t v;memcpy(&v,data+i*4,4);n=v;}if(n>max)max=n;}
            if(max<4096) {
                int vertex_enabled=enabled(0x8074),uv_enabled=enabled(0x8078),color_enabled=enabled(0x8076);
                foundation_graphics_state(stream,texture);
                fprintf(stream,"{\"op\":\"draw\",\"frame\":%u,\"texture\":%u,\"program\":%u,\"textureEpoch\":%u,\"textureSize\":[%d,%d],\"framebuffer\":%u,\"renderTargetTexture\":%u,\"cacheDraw\":%s,\"mode\":%u,\"count\":%d,\"vertexSize\":%d,\"uvSize\":%d,\"indexType\":%u,\"indices\":[",frame_number,texture,program,texture<65536?texture_epoch[texture]:0,texture<65536?texture_width[texture]:0,texture<65536?texture_height[texture]:0,framebuffer,attachment,cache_draw?"true":"false",mode,count,vertex_array.size,uv_array.size,type);
                for(int i=0;i<count;i++) { unsigned n=0;if(width==1)n=data[i];else if(width==2){uint16_t v;memcpy(&v,data+i*2,2);n=v;}else{uint32_t v;memcpy(&v,data+i*4,4);n=v;}fprintf(stream,"%s%u",i?",":"",n); }
                fputs("],\"vertices\":[",stream);
                for(unsigned n=0;n<=max;n++) {
                    /* Disabled client arrays are not submitted graphics data.
                     * Reading their stale pointer values can expose unrelated
                     * bytes and produce meaningless NaNs. */
                    double v[4]={0},uv[4]={0},c[4]={0};
                    int ok=vertex_enabled&&components(vertex_array,n,v);
                    int uvok=uv_enabled&&components(uv_array,n,uv);
                    int cok=color_enabled&&components(color_array,n,c);
                    fprintf(stream,"%s{\"id\":%u,\"ok\":%s,\"xy\":",n?",":"",n,ok?"true":"false");numbers(v,2);
                    fprintf(stream,",\"xyFinite\":%s,\"uv\":",finite_components(v,2)?"true":"false");numbers(uv,2);
                    fprintf(stream,",\"uvOk\":%s,\"uvFinite\":%s,\"color\":",uvok?"true":"false",finite_components(uv,2)?"true":"false");numbers(c,4);
                    fprintf(stream,",\"colorOk\":%s,\"colorFinite\":%s}",cok?"true":"false",finite_components(c,4)?"true":"false");
                }
                fprintf(stream,"]}\n");
            }
        }
        free(data);
    }
    real(mode,count,type,indices);
    snapshot_draw_pixels("after",serial);
}
static void snapshot_used_textures(void){
    void(*bind)(GLenum,GLuint)=dlsym(RTLD_NEXT,"glBindTexture");
    void(*get)(GLenum,GLint,GLenum,GLenum,void*)=dlsym(RTLD_NEXT,"glGetTexImage");
    if(!get || binding(0x88ed) || binding(0x0D02) || binding(0x0D03) || binding(0x0D04))return;
    GLuint previous=binding(0x8069);
    for(unsigned id=1;id<65536;id++)if(used_texture[id]){
        used_texture[id]=0;int w=texture_width[id],h=texture_height[id];
        if(w<=0||h<=0||w>2048||h>2048)continue;
        unsigned char *data=malloc((size_t)w*h*4);if(!data)continue;
        bind(0x0DE1,id);get(0x0DE1,0,GL_RGBA,GL_UNSIGNED_BYTE,data);
        char path[2048];snprintf(path,sizeof(path),"%s/texture-frame-%u-epoch-%u-%u-%dx%d.rgba",directory,frame_number,texture_epoch[id],id,w,h);
        FILE *image=fopen(path,"wb");if(image){fwrite(data,4,(size_t)w*h,image);fclose(image);fprintf(stream,"{\"op\":\"textureSnapshot\",\"id\":%u,\"epoch\":%u,\"frame\":%u,\"width\":%d,\"height\":%d,\"rawPath\":\"%s\",\"scope\":\"public GL rendered texture pixels\"}\n",id,texture_epoch[id],frame_number,w,h,path);}free(data);
    }
    bind(0x0DE1,previous);
}

void glXSwapBuffers(void *display,unsigned long drawable) {
    void (*real)(void*,unsigned long)=dlsym(RTLD_NEXT,"glXSwapBuffers"); real(display,drawable);init();
    if(active && stream) { if(snapshot_requested)snapshot_used_textures();else memset(used_texture,0,sizeof(used_texture));struct timespec now;clock_gettime(CLOCK_MONOTONIC,&now);fprintf(stream,"{\"op\":\"frameEnd\",\"frame\":%u,\"monotonicNs\":%llu}\n",frame_number,(unsigned long long)now.tv_sec*1000000000ull+now.tv_nsec);active=0;if(snapshot_requested && getenv("ISAAC_GL_TRACE_STOP_AFTER_FRAME"))raise(SIGSTOP); }
    if(stream && control) {
        FILE *request=fopen(control,"r");
        if(request) { if(fgets(label,sizeof(label),request)){label[strcspn(label,"\r\n")]=0;active=1;snapshot_requested=1;frame_number++;draw_number=0;fprintf(stream,"{\"op\":\"frameStart\",\"frame\":%u,\"label\":\"%s\",\"simulationTick\":null}\n",frame_number,label);}fclose(request);unlink(control); }
        else if(getenv("ISAAC_GL_TRACE_EVERY_FRAME")){active=1;snapshot_requested=0;frame_number++;draw_number=0;fprintf(stream,"{\"op\":\"frameStart\",\"frame\":%u,\"label\":\"every\",\"simulationTick\":null}\n",frame_number);}
    }
}

/* Optional observations of libc's documented RNG ABI. Forward the original
 * result unchanged; these records are diagnostics, never a game input. */
int rand(void){
    static __thread int(*real)(void);static _Atomic unsigned long long calls;
    if(!real)real=dlsym(RTLD_NEXT,"rand");
    int value=real();
    if(getenv("ISAAC_GL_TRACE_LIBC_RAND")){init();if(stream)fprintf(stream,"{\"op\":\"libcRand\",\"draw\":%llu,\"value\":%d,\"graphicsFrame\":%u,\"graphicsActive\":%s}\n",++calls,value,frame_number,active?"true":"false");}
    return value;
}

/* Uniform names/values are submitted graphics API data, never shader source. */
static void uniform_floats(GLint location,const GLfloat *values,int size,int array) {
    if(!active||!stream)return;
    fprintf(stream,"{\"op\":\"%s\",\"program\":%u,\"location\":%d,\"values\":[",array?"uniformArray":"uniform",program,location);
    for(int i=0;i<size;i++){if(i)fputc(',',stream);number(values[i]);}
    fputs("]}\n",stream);
}
void glUseProgram(GLuint id) {
    void (*real)(GLuint)=dlsym(RTLD_NEXT,"glUseProgram"); real(id); program=id;
    if(active && stream) fprintf(stream,"{\"op\":\"program\",\"id\":%u}\n",id);
}
GLint glGetUniformLocation(GLuint id,const char *name) {
    GLint (*real)(GLuint,const char*)=dlsym(RTLD_NEXT,"glGetUniformLocation"); GLint location=real(id,name);init();
    int seen=0;for(unsigned i=0;i<uniform_name_count;i++)if(uniform_names[i].program==id && uniform_names[i].location==location && !strcmp(uniform_names[i].name,name))seen=1;
    if(!seen && stream){fprintf(stream,"{\"op\":\"uniformName\",\"program\":%u,\"location\":%d,\"name\":\"%s\"}\n",id,location,name);if(uniform_name_count<256){struct UniformName *entry=&uniform_names[uniform_name_count++];entry->program=id;entry->location=location;snprintf(entry->name,sizeof(entry->name),"%s",name);}}
    return location;
}
void glUniform1f(GLint loc,GLfloat a) {
    void (*real)(GLint,GLfloat)=dlsym(RTLD_NEXT,"glUniform1f");real(loc,a);
    uniform_floats(loc,&a,1,0);
}
void glUniform2f(GLint loc,GLfloat a,GLfloat b) {
    void (*real)(GLint,GLfloat,GLfloat)=dlsym(RTLD_NEXT,"glUniform2f");real(loc,a,b);
    GLfloat values[]={a,b};uniform_floats(loc,values,2,0);
}
void glUniform3f(GLint loc,GLfloat a,GLfloat b,GLfloat c) {
    void (*real)(GLint,GLfloat,GLfloat,GLfloat)=dlsym(RTLD_NEXT,"glUniform3f");real(loc,a,b,c);
    GLfloat values[]={a,b,c};uniform_floats(loc,values,3,0);
}
void glUniform4f(GLint loc,GLfloat a,GLfloat b,GLfloat c,GLfloat d) {
    void (*real)(GLint,GLfloat,GLfloat,GLfloat,GLfloat)=dlsym(RTLD_NEXT,"glUniform4f");real(loc,a,b,c,d);
    GLfloat values[]={a,b,c,d};uniform_floats(loc,values,4,0);
}
void glUniform1i(GLint loc,GLint a) {
    void (*real)(GLint,GLint)=dlsym(RTLD_NEXT,"glUniform1i");real(loc,a);
    if(active&&stream)fprintf(stream,"{\"op\":\"uniformInt\",\"program\":%u,\"location\":%d,\"values\":[%d]}\n",program,loc,a);
}
void glUniform1fv(GLint loc,GLsizei count,const GLfloat *a) {
    void (*real)(GLint,GLsizei,const GLfloat*)=dlsym(RTLD_NEXT,"glUniform1fv");real(loc,count,a);
    if(count>0&&count<=16)uniform_floats(loc,a,count,1);
}
void glUniform2fv(GLint loc,GLsizei count,const GLfloat *a) {
    void (*real)(GLint,GLsizei,const GLfloat*)=dlsym(RTLD_NEXT,"glUniform2fv");real(loc,count,a);
    if(count>0&&count<=16)uniform_floats(loc,a,count*2,1);
}
void glUniform3fv(GLint loc,GLsizei count,const GLfloat *a) {
    void (*real)(GLint,GLsizei,const GLfloat*)=dlsym(RTLD_NEXT,"glUniform3fv");real(loc,count,a);
    if(count>0&&count<=16)uniform_floats(loc,a,count*3,1);
}
void glUniform4fv(GLint loc,GLsizei count,const GLfloat *a) {
    void (*real)(GLint,GLsizei,const GLfloat*)=dlsym(RTLD_NEXT,"glUniform4fv");real(loc,count,a);
    if(count>0&&count<=16)uniform_floats(loc,a,count*4,1);
}
typedef void (*GLXextFunc)(void);
static GLXextFunc intercepted(const GLubyte *name,GLXextFunc real) {
    if(!real)return NULL;
#define HOOK(symbol) if(!strcmp((const char*)name,#symbol))return (GLXextFunc)symbol
    HOOK(glUseProgram); HOOK(glGetUniformLocation); HOOK(glUniform1f); HOOK(glUniform2f); HOOK(glUniform3f); HOOK(glUniform4f);
    HOOK(glUniform1i);HOOK(glUniform1fv);HOOK(glUniform2fv);HOOK(glUniform3fv);HOOK(glUniform4fv);
    HOOK(glVertexPointer);HOOK(glTexCoordPointer);HOOK(glColorPointer);HOOK(glDrawElements);HOOK(glBindTexture);HOOK(glTexImage2D);HOOK(glTexSubImage2D);
#undef HOOK
    return real;
}
GLXextFunc glXGetProcAddress(const GLubyte *name) {
    GLXextFunc (*real)(const GLubyte*)=dlsym(RTLD_NEXT,"glXGetProcAddress");return intercepted(name,real(name));
}
GLXextFunc glXGetProcAddressARB(const GLubyte *name) {
    GLXextFunc (*real)(const GLubyte*)=dlsym(RTLD_NEXT,"glXGetProcAddressARB");return intercepted(name,real(name));
}
