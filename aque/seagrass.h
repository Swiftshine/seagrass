#pragma once

#include <stddef.h>

typedef struct SGRuntime SGRuntime;

// Runtime functions
struct SGRuntime* sg_create_runtime(const char* pScriptPath);
void sg_free_runtime(SGRuntime* pRuntime);
void sg_execute_runtime(SGRuntime* pRuntime);
void* sg_get_serialized_bytes(SGRuntime* pRuntime, const char* pTargetName);
size_t sg_get_serialized_size(SGRuntime* pRuntime, const char* pTargetName);
