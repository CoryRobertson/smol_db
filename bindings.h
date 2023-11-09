#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>


typedef struct FFISmolDBClient FFISmolDBClient;

void smol_db_client_free(struct FFISmolDBClient *client_ptr);

struct FFISmolDBClient *smol_db_client_new(const char *ip);

void smol_db_client_set_key(struct FFISmolDBClient *client_ptr, const char *key_ptr);

uint8_t *smol_db_client_write_db(struct FFISmolDBClient *client_ptr,
                                 const char *name,
                                 const char *location,
                                 const char *data);
