#include <stdbool.h>
bool vxkey_native_start(const char *root);
char *vxkey_native_tick(const char *status);
void vxkey_native_free(char *value);

bool vxkey_native_engine_faulted(void);
