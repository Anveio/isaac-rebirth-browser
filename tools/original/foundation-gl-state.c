/* Read-only observations of the documented graphics API state. */
#define _GNU_SOURCE
#include <dlfcn.h>
#include <math.h>
#include <stdio.h>
#include <string.h>

typedef unsigned int GLenum;
typedef unsigned int GLuint;
typedef int GLint;
typedef int GLsizei;
typedef float GLfloat;
typedef unsigned char GLboolean;

static void float_json(FILE *stream, GLfloat value) {
    if (isfinite(value)) fprintf(stream, "%.9g", value);
    else fputs("null", stream);
}

static void integers(FILE *stream, const char *name, GLenum parameter, int length) {
    void (*get)(GLenum, GLint *) = dlsym(RTLD_NEXT, "glGetIntegerv");
    GLint values[16] = {0};
    if (!get) return;
    get(parameter, values);
    fprintf(stream, ",\"%s\":[", name);
    for (int i = 0; i < length; i++) fprintf(stream, "%s%d", i ? "," : "", values[i]);
    fputc(']', stream);
}

static void floats(FILE *stream, const char *name, GLenum parameter, int length) {
    void (*get)(GLenum, GLfloat *) = dlsym(RTLD_NEXT, "glGetFloatv");
    GLfloat values[16] = {0};
    if (!get) return;
    get(parameter, values);
    fprintf(stream, ",\"%s\":[", name);
    for (int i = 0; i < length; i++) { if (i) fputc(',', stream); float_json(stream, values[i]); }
    fputc(']', stream);
}

static void depth_attachment(FILE *stream, GLint framebuffer) {
    void (*attachment)(GLenum, GLenum, GLenum, GLint *) = dlsym(RTLD_NEXT, "glGetFramebufferAttachmentParameteriv");
    fprintf(stream, ",\"depthAttachment\":{\"framebuffer\":%d", framebuffer);
    if (!framebuffer) {
        fputs(",\"kind\":\"default\"}", stream);
        return;
    }
    if (!attachment) { fputs(",\"available\":false}", stream); return; }
    GLint type = 0, object = 0;
    attachment(0x8D40, 0x8D00, 0x8CD0, &type); /* FRAMEBUFFER, DEPTH_ATTACHMENT, OBJECT_TYPE */
    if (type) attachment(0x8D40, 0x8D00, 0x8CD1, &object); /* OBJECT_NAME */
    fprintf(stream, ",\"available\":true,\"objectType\":%d,\"object\":%d", type, object);
    if (type == 0x8D41 && object > 0) { /* RENDERBUFFER */
        const unsigned char *(*version)(GLenum) = dlsym(RTLD_NEXT, "glGetString");
        void (*named_parameter)(GLuint, GLenum, GLint *) = dlsym(RTLD_NEXT, "glGetNamedRenderbufferParameteriv");
        const unsigned char *description = version ? version(0x1F02) : NULL; /* VERSION */
        int major = 0, minor = 0;
        if (description) (void)sscanf((const char *)description, "%d.%d", &major, &minor);
        /* Named queries preserve all bindings. Older contexts retain identity
           and depth bits above, without invoking unsupported DSA entry points. */
        int available = named_parameter && (major > 4 || (major == 4 && minor >= 5));
        fprintf(stream, ",\"renderbufferParametersAvailable\":%s", available ? "true" : "false");
        if (available) {
            const struct { const char *name; GLenum parameter; } fields[] = {
                {"width", 0x8D42}, {"height", 0x8D43}, {"internalFormat", 0x8D44},
                {"depthBits", 0x8D54}, {"stencilBits", 0x8D55}, {"samples", 0x8CAB}
            };
            for (unsigned i = 0; i < sizeof(fields) / sizeof(*fields); i++) {
                GLint value = 0;
                named_parameter((GLuint)object, fields[i].parameter, &value);
                fprintf(stream, ",\"%s\":%d", fields[i].name, value);
            }
        }
    }
    fputc('}', stream);
}

void foundation_graphics_state(FILE *stream, unsigned texture) {
    void (*get)(GLenum, GLint *) = dlsym(RTLD_NEXT, "glGetIntegerv");
    GLboolean (*enabled)(GLenum) = dlsym(RTLD_NEXT, "glIsEnabled");
    void (*texture_parameter)(GLenum, GLenum, GLint *) = dlsym(RTLD_NEXT, "glGetTexParameteriv");
    void (*texture_level)(GLenum, GLint, GLenum, GLint *) = dlsym(RTLD_NEXT, "glGetTexLevelParameteriv");
    if (!stream || !get) return;
    fprintf(stream, "{\"op\":\"graphicsState\",\"texture\":%u", texture);
    integers(stream, "viewport", 0x0BA2, 4);
    integers(stream, "scissorBox", 0x0C10, 4);
    integers(stream, "blendSourceRGB", 0x80C9, 1);
    integers(stream, "blendDestinationRGB", 0x80C8, 1);
    integers(stream, "blendSourceAlpha", 0x80CB, 1);
    integers(stream, "blendDestinationAlpha", 0x80CA, 1);
    integers(stream, "blendEquationRGB", 0x8009, 1);
    integers(stream, "blendEquationAlpha", 0x883D, 1);
    integers(stream, "colorWriteMask", 0x0C23, 4);
    integers(stream, "activeTexture", 0x84E0, 1);
    integers(stream, "alphaTestFunction", 0x0BC1, 1);
    integers(stream, "depthFunction", 0x0B74, 1);
    integers(stream, "depthWriteMask", 0x0B72, 1);
    integers(stream, "depthBits", 0x0D56, 1);
    GLint framebuffer = 0;
    get(0x8CA6, &framebuffer); /* FRAMEBUFFER_BINDING */
    depth_attachment(stream, framebuffer);
    integers(stream, "sampleBuffers", 0x80A8, 1);
    integers(stream, "samples", 0x80A9, 1);
    integers(stream, "subpixelBits", 0x0D50, 1);
    integers(stream, "redBits", 0x0D52, 1);
    integers(stream, "greenBits", 0x0D53, 1);
    integers(stream, "blueBits", 0x0D54, 1);
    integers(stream, "alphaBits", 0x0D55, 1);
    floats(stream, "modelView", 0x0BA6, 16);
    floats(stream, "projection", 0x0BA7, 16);
    floats(stream, "textureMatrix", 0x0BA8, 16);
    floats(stream, "blendColor", 0x8005, 4);
    floats(stream, "alphaTestReference", 0x0BC2, 1);
    floats(stream, "depthClearValue", 0x0B73, 1);
    floats(stream, "depthRange", 0x0B70, 2);
    floats(stream, "currentColor", 0x0B00, 4);
    floats(stream, "currentTextureCoordinates", 0x0B03, 4);
    if (enabled) {
        const struct { const char *name; GLenum value; } capabilities[] = {
            {"blendEnabled", 0x0BE2}, {"scissorEnabled", 0x0C11}, {"ditherEnabled", 0x0BD0},
            {"depthEnabled", 0x0B71}, {"stencilEnabled", 0x0B90}, {"alphaTestEnabled", 0x0BC0},
            {"multisampleEnabled", 0x809D}, {"framebufferSRGBEnabled", 0x8DB9}
        };
        for (unsigned i = 0; i < sizeof(capabilities) / sizeof(*capabilities); i++)
            fprintf(stream, ",\"%s\":%s", capabilities[i].name, enabled(capabilities[i].value) ? "true" : "false");
    }
    if (texture_parameter) {
        const struct { const char *name; GLenum value; } parameters[] = {
            {"minFilter", 0x2801}, {"magFilter", 0x2800}, {"wrapS", 0x2802}, {"wrapT", 0x2803}
        };
        for (unsigned i = 0; i < sizeof(parameters) / sizeof(*parameters); i++) {
            GLint value = 0;
            texture_parameter(0x0DE1, parameters[i].value, &value);
            fprintf(stream, ",\"%s\":%d", parameters[i].name, value);
        }
    }
    if (texture_level) {
        GLint internal = 0, compressed = 0;
        texture_level(0x0DE1, 0, 0x1003, &internal);
        texture_level(0x0DE1, 0, 0x86A1, &compressed);
        fprintf(stream, ",\"textureInternalFormat\":%d,\"textureCompressed\":%s", internal, compressed ? "true" : "false");
    }

    /* Uniform values are graphics inputs. This does not read shader source. */
    GLint program = 0;
    get(0x8B8D, &program);
    void (*program_parameter)(GLuint, GLenum, GLint *) = dlsym(RTLD_NEXT, "glGetProgramiv");
    void (*uniform_description)(GLuint, GLuint, GLsizei, GLsizei *, GLint *, GLenum *, char *) = dlsym(RTLD_NEXT, "glGetActiveUniform");
    GLint (*uniform_location)(GLuint, const char *) = dlsym(RTLD_NEXT, "glGetUniformLocation");
    void (*uniform_float)(GLuint, GLint, GLfloat *) = dlsym(RTLD_NEXT, "glGetUniformfv");
    void (*uniform_integer)(GLuint, GLint, GLint *) = dlsym(RTLD_NEXT, "glGetUniformiv");
    fprintf(stream, ",\"program\":%d,\"uniforms\":[", program);
    if (program && program_parameter && uniform_description && uniform_location && uniform_float && uniform_integer) {
        GLint count = 0;
        program_parameter(program, 0x8B86, &count);
        if (count > 0 && count <= 256) {
            int emitted = 0;
            for (GLint index = 0; index < count; index++) {
                char name[256] = {0}; GLsizei length = 0; GLint size = 0; GLenum type = 0;
                uniform_description(program, index, sizeof(name), &length, &size, &type, name);
                if (size < 1 || size > 16 || length < 1 || length >= (GLsizei)sizeof(name)) continue;
                int width = 0, is_integer = 0;
                switch (type) {
                    case 0x1406: width = 1; break;
                    case 0x8B50: width = 2; break;
                    case 0x8B51: width = 3; break;
                    case 0x8B52: width = 4; break;
                    case 0x8B5A: width = 4; break;
                    case 0x8B5B: width = 9; break;
                    case 0x8B5C: width = 16; break;
                    case 0x1404: case 0x8B56: case 0x8B5E: case 0x8B60: width = 1; is_integer = 1; break;
                    default: continue;
                }
                GLint location = uniform_location(program, name);
                if (location < 0) continue;
                GLfloat values[256] = {0}; GLint ivals[256] = {0};
                for (int element = 0; element < size; element++) {
                    GLint element_location = location;
                    if (size > 1) {
                        char element_name[320];
                        size_t base_length = strlen(name);
                        if (base_length >= 3 && !strcmp(name + base_length - 3, "[0]")) base_length -= 3;
                        snprintf(element_name, sizeof(element_name), "%.*s[%d]", (int)base_length, name, element);
                        element_location = uniform_location(program, element_name);
                    }
                    if (element_location < 0) continue;
                    if (is_integer) uniform_integer(program, element_location, ivals + element * width);
                    else uniform_float(program, element_location, values + element * width);
                }
                fprintf(stream, "%s{\"name\":\"%s\",\"type\":%u,\"size\":%d,\"location\":%d,\"values\":[", emitted++ ? "," : "", name, type, size, location);
                for (int i = 0; i < width * size; i++) {
                    if (is_integer) fprintf(stream, "%s%d", i ? "," : "", ivals[i]);
                    else { if (i) fputc(',', stream); float_json(stream, values[i]); }
                }
                fprintf(stream, "]}");
            }
        }
    }
    fprintf(stream, "]}\n");
}
