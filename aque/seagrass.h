#pragma once

typedef struct SGRuntime SGRuntime;

// Runtime functions
struct SGRuntime* sg_create_runtime(const char* pScriptPath);
void sg_free_runtime(SGRuntime* pRuntime);
void sg_execute_runtime(SGRuntime* pRuntime);
void* sg_get_serialized_result(SGRuntime* pRuntime);
