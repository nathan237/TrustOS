// metal_shader_fuzzer.m - Metal Shader Fuzzer targeting AGXCompilerService
// Generates malformed Metal shaders to trigger bugs in the OOP-JIT compiler
// Compile: clang -framework Metal -framework MetalKit -o sfuzz metal_shader_fuzzer.m

#import <Metal/Metal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Generate a craft Metal shader source with potential edge cases
const char* generate_fuzz_shader(int seed) {
    static char shader[4096];

    // Various malformation strategies
    switch (seed % 8) {
        case 0: // Extremely large array
            snprintf(shader, sizeof(shader),
                "#include <metal_stdlib>\n"
                "using namespace metal;\n"
                "kernel void fuzz(device float *out [[buffer(0)]],\n"
                "                 uint id [[thread_position_in_grid]]) {\n"
                "    float arr[%d];\n"  // stack overflow in compiler?
                "    for (int i = 0; i < %d; i++) arr[i] = float(i);\n"
                "    out[id] = arr[id %% %d];\n"
                "}\n",
                1000000 + (seed * 1000), 1000000 + (seed * 1000),
                1000000 + (seed * 1000));
            break;

        case 1: // Deep recursion via function calls
            snprintf(shader, sizeof(shader),
                "#include <metal_stdlib>\n"
                "using namespace metal;\n"
                "float f0(float x) { return x * 1.1; }\n"
                "float f1(float x) { return f0(f0(x)); }\n"
                "float f2(float x) { return f1(f1(x)); }\n"
                "float f3(float x) { return f2(f2(x)); }\n"
                "float f4(float x) { return f3(f3(x)); }\n"
                "float f5(float x) { return f4(f4(x)); }\n"
                "float f6(float x) { return f5(f5(x)); }\n"
                "float f7(float x) { return f6(f6(x)); }\n"
                "kernel void fuzz(device float *out [[buffer(0)]],\n"
                "                 uint id [[thread_position_in_grid]]) {\n"
                "    out[id] = f7(float(id));\n"
                "}\n");
            break;

        case 2: // Thread group memory abuse
            snprintf(shader, sizeof(shader),
                "#include <metal_stdlib>\n"
                "using namespace metal;\n"
                "kernel void fuzz(device float *out [[buffer(0)]],\n"
                "                 uint id [[thread_position_in_grid]],\n"
                "                 uint lid [[thread_position_in_threadgroup]]) {\n"
                "    threadgroup float shared[%d];\n"  // huge threadgroup mem
                "    shared[lid] = float(id);\n"
                "    threadgroup_barrier(mem_flags::mem_threadgroup);\n"
                "    out[id] = shared[lid %% %d];\n"
                "}\n",
                65536 + seed, 65536 + seed);
            break;

        case 3: // Texture access with extreme coordinates
            snprintf(shader, sizeof(shader),
                "#include <metal_stdlib>\n"
                "using namespace metal;\n"
                "kernel void fuzz(texture2d<float, access::read> tex [[texture(0)]],\n"
                "                 device float *out [[buffer(0)]],\n"
                "                 uint id [[thread_position_in_grid]]) {\n"
                "    float4 val = tex.read(uint2(%u, %u));\n"
                "    out[id] = val.x;\n"
                "}\n",
                0xFFFFFFFF - seed, 0xFFFFFFFF - seed);
            break;

        case 4: // Atomic operations stress
            snprintf(shader, sizeof(shader),
                "#include <metal_stdlib>\n"
                "using namespace metal;\n"
                "kernel void fuzz(device atomic_uint *counter [[buffer(0)]],\n"
                "                 uint id [[thread_position_in_grid]]) {\n"
                "    for (int i = 0; i < %d; i++) {\n"
                "        atomic_fetch_add_explicit(counter, 1, memory_order_relaxed);\n"
                "    }\n"
                "}\n",
                100000 + seed);
            break;

        case 5: // Indirect buffer access (buffer of pointers)
            snprintf(shader, sizeof(shader),
                "#include <metal_stdlib>\n"
                "using namespace metal;\n"
                "struct Args { device float *ptr; uint size; };\n"
                "kernel void fuzz(device Args *args [[buffer(0)]],\n"
                "                 device float *out [[buffer(1)]],\n"
                "                 uint id [[thread_position_in_grid]]) {\n"
                "    out[id] = args[id %% %d].ptr[id %% args[0].size];\n"
                "}\n",
                seed + 1);
            break;

        default: // Normal valid shader (control)
            snprintf(shader, sizeof(shader),
                "#include <metal_stdlib>\n"
                "using namespace metal;\n"
                "kernel void fuzz(device float *in [[buffer(0)]],\n"
                "                 device float *out [[buffer(1)]],\n"
                "                 uint id [[thread_position_in_grid]]) {\n"
                "    out[id] = in[id] * 2.0 + 1.0;\n"
                "}\n");
            break;
    }
    return shader;
}

int main(int argc, char **argv) {
    printf("[*] Metal Shader Fuzzer for AGXCompilerService\n");

    @autoreleasepool {
        id<MTLDevice> device = MTLCreateSystemDefaultDevice();
        if (!device) {
            printf("[-] No Metal device\n");
            return 1;
        }
        printf("[+] Device: %s\n", [[device name] UTF8String]);

        int iterations = argc > 1 ? atoi(argv[1]) : 1000;
        int crashes = 0;

        for (int i = 0; i < iterations; i++) {
            @autoreleasepool {
                const char *src = generate_fuzz_shader(i);
                NSString *source = [NSString stringWithUTF8String:src];
                NSError *error = nil;

                id<MTLLibrary> lib = [device newLibraryWithSource:source
                                                          options:nil
                                                            error:&error];
                if (error) {
                    // Compiler error = expected for malformed shaders
                    // Crash or hang = interesting!
                    if ([[error localizedDescription] containsString:@"internal"]) {
                        printf("[!!!] Internal compiler error at seed %d!\n", i);
                        crashes++;
                    }
                } else {
                    // Compiled successfully - try to create pipeline
                    id<MTLFunction> func = [lib newFunctionWithName:@"fuzz"];
                    if (func) {
                        NSError *pipeErr = nil;
                        id<MTLComputePipelineState> pipeline =
                            [device newComputePipelineStateWithFunction:func
                                                                 error:&pipeErr];
                        if (pipeErr) {
                            printf("[!] Pipeline error at seed %d: %s\n",
                                   i, [[pipeErr localizedDescription] UTF8String]);
                        }
                    }
                }

                if (i % 100 == 0) {
                    printf("[*] Progress: %d/%d (crashes: %d)\n", i, iterations, crashes);
                }
            }
        }

        printf("[*] Done: %d iterations, %d internal errors\n", iterations, crashes);
    }
    return 0;
}