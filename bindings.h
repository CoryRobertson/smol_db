#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>


#define DATA_NOT_FOUND_STATE 2

#define ERROR_STATE 1

#define OK_STATE 0

typedef struct FFISmolDBClient FFISmolDBClient;

int32_t smol_db_client_disconnect(struct FFISmolDBClient *client_ptr);

void smol_db_client_free(struct FFISmolDBClient *client_ptr);

struct FFISmolDBClient *smol_db_client_new(const char *ip);

const char *smol_db_client_read_db(struct FFISmolDBClient *client_ptr,
                                   const char *name,
                                   const char *location);

int32_t smol_db_client_reconnect(struct FFISmolDBClient *client_ptr);

int32_t smol_db_client_set_key(struct FFISmolDBClient *client_ptr, const char *key_ptr);

int32_t smol_db_client_setup_encryption(struct FFISmolDBClient *client_ptr);

const char *smol_db_client_write_db(struct FFISmolDBClient *client_ptr,
                                    const char *name,
                                    const char *location,
                                    const char *data);
